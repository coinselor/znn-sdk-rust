//! The WebSocket JSON-RPC client.
//!
//! [`WsClient`] wraps a `jsonrpsee` WebSocket client and exposes the status
//! lifecycle plus [`Client::send_request`]. Before a successful [`initialize`],
//! requests fail fast with [`ClientError::NoConnection`]; once connected they
//! dispatch through the live transport.
//!
//! `WsClient` also opens websocket-backed `ledger.subscribe` streams through
//! jsonrpsee's subscription support. A compatibility bridge listens for
//! `ledger.subscription` method notifications and routes them to the handles
//! created by [`SubscribeApi`](crate::api::subscribe::SubscribeApi). Calling
//! [`stop`] closes active handles; calling [`initialize`] again does not
//! resubscribe closed handles, so callers must create new handles after a stop.
//!
//! [`initialize`]: WsClient::initialize
//! [`stop`]: WsClient::stop

use crate::api::subscribe::Subscription;
use crate::client::dial::{ConnectionState, WS_SCHEMES, dial, validate_url};
use crate::client::exceptions::ClientError;
use crate::client::interfaces::Client;
use jsonrpsee::core::client::{ClientT, SubscriptionClientT, SubscriptionKind};
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::types::SubscriptionId;
use jsonrpsee::ws_client::{WsClient as RpcClient, WsClientBuilder};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const LEDGER_SUBSCRIBE_METHOD: &str = "ledger.subscribe";
const LEDGER_UNSUBSCRIBE_METHOD: &str = "ledger.unsubscribe";
const LEDGER_SUBSCRIPTION_METHOD: &str = "ledger.subscription";

type SubscriptionRegistry = HashMap<String, mpsc::UnboundedSender<Value>>;
type SharedSubscriptionRegistry = Arc<Mutex<SubscriptionRegistry>>;

/// A JSON-RPC client over WebSocket.
pub struct WsClient {
    state: ConnectionState,
    client: Option<RpcClient>,
    subscriptions: SharedSubscriptionRegistry,
    notification_task: Option<JoinHandle<()>>,
    client_stopped: Arc<AtomicBool>,
}

impl Default for WsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WsClient {
    /// Creates a client in the `Uninitialized` state.
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Uninitialized,
            client: None,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            notification_task: None,
            client_stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    fn subscription_registry(&self) -> MutexGuard<'_, SubscriptionRegistry> {
        self.subscriptions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Returns `true` when there is no live transport.
    pub fn is_closed(&self) -> bool {
        self.state != ConnectionState::Running
    }

    /// Returns the intended connection state.
    pub fn status(&self) -> ConnectionState {
        self.state
    }

    /// Opens a `ledger.subscribe` websocket subscription and returns a handle
    /// that receives notifications for the server-assigned subscription id.
    pub(crate) async fn subscribe_stream(
        &self,
        params: &[Value],
    ) -> Result<Subscription, crate::error::Error> {
        let topic = subscription_topic_from_params(params)?;
        let client = self.client.as_ref().ok_or(ClientError::NoConnection)?;
        if self.state != ConnectionState::Running {
            return Err(ClientError::NoConnection.into());
        }

        let mut rpc_params = ArrayParams::new();
        for value in params {
            rpc_params
                .insert(value.clone())
                .map_err(|_| ClientError::NoConnection)?;
        }

        let rpc_subscription = client
            .subscribe::<Value, _>(
                LEDGER_SUBSCRIBE_METHOD,
                rpc_params,
                LEDGER_UNSUBSCRIBE_METHOD,
            )
            .await
            .map_err(|err| crate::error::Error::InvalidInput(err.to_string()))?;
        let id = subscription_id_from_kind(rpc_subscription.kind())?;
        let (sender, receiver) = mpsc::unbounded_channel();
        self.subscription_registry().insert(id.clone(), sender);
        let cleanup_id = id.clone();
        let cleanup_registry = Arc::clone(&self.subscriptions);
        let registry_cleanup = Box::new(move || {
            cleanup_registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&cleanup_id);
        });
        Ok(Subscription::from_rpc(
            id,
            topic,
            rpc_subscription,
            receiver,
            Some(registry_cleanup),
            Arc::clone(&self.client_stopped),
        ))
    }

    /// Stops the client and drops the live transport.
    ///
    /// All registered subscription senders are dropped, so active handles close
    /// gracefully and their receivers finish with `Ok(None)`.
    pub fn stop(&mut self) {
        self.state = ConnectionState::Stopped;
        self.client_stopped.store(true, Ordering::Release);
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
        self.client = None;
        self.subscription_registry().clear();
    }

    /// Dials `url`, setting the client to `Running` on success. When `retry` is
    /// set, the dial is attempted up to [`NUM_RETRIES`](crate::client::constants::NUM_RETRIES)
    /// times with a short backoff between attempts; a malformed URL yields
    /// [`ClientError::NoConnection`] immediately.
    pub async fn initialize(&mut self, url: &str, retry: bool) -> Result<bool, ClientError> {
        // `dial`'s `connect_one` bound (`Fn(&str) -> Fut`) requires the returned
        // future to be independent of the borrowed `&str`, so the connect step
        // captures an owned copy of the URL rather than borrowing the argument.
        let connect_url = url.to_string();
        let this = self;
        dial(
            &mut this.state,
            url,
            retry,
            |url| validate_url(url, WS_SCHEMES),
            move |_: &str| {
                let connect_url = connect_url.clone();
                async move {
                    let client = WsClientBuilder::default()
                        .build(&connect_url)
                        .await
                        .map_err(|_| ClientError::NoConnection)?;
                    let notifications = client
                        .subscribe_to_method::<Value>(LEDGER_SUBSCRIPTION_METHOD)
                        .await
                        .map_err(|_| ClientError::NoConnection)?;
                    Ok((client, notifications))
                }
            },
            |(client, notifications)| {
                if let Some(task) = this.notification_task.take() {
                    task.abort();
                }
                this.notification_task = Some(spawn_notification_router(
                    Arc::clone(&this.subscriptions),
                    notifications,
                ));
                this.client = Some(client);
                // Clear any prior stop flag so closures observed on handles
                // opened after this (re)connect classify as transport events
                // rather than a stale `ClientStopped`.
                this.client_stopped.store(false, Ordering::Release);
            },
        )
        .await
    }
}

fn spawn_notification_router(
    registry: SharedSubscriptionRegistry,
    mut notifications: jsonrpsee::core::client::Subscription<Value>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(next) = notifications.next().await {
            let Ok(value) = next else {
                continue;
            };
            if let Some((id, payload)) = parse_subscription_notification(value) {
                deliver_subscription_notification(&registry, &id, payload);
            }
        }
        registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    })
}

fn deliver_subscription_notification(
    registry: &SharedSubscriptionRegistry,
    id: &str,
    payload: Value,
) {
    let mut registry = registry
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let receiver_dropped = match registry.get(id) {
        Some(sender) => sender.send(payload).is_err(),
        None => return,
    };
    if receiver_dropped {
        registry.remove(id);
    }
}

fn parse_subscription_notification(value: Value) -> Option<(String, Value)> {
    match value {
        Value::Object(mut object) => {
            let id = object
                .remove("subscription")
                .or_else(|| object.remove("id"))
                .and_then(subscription_id_value_to_string)?;
            let payload = object
                .remove("result")
                .or_else(|| object.remove("data"))
                .or_else(|| object.remove("payload"))
                .unwrap_or(Value::Object(object));
            Some((id, payload))
        }
        Value::Array(mut values) if values.len() == 2 => {
            let payload = values.pop()?;
            let id = values.pop().and_then(subscription_id_value_to_string)?;
            Some((id, payload))
        }
        _ => None,
    }
}

fn subscription_topic_from_params(params: &[Value]) -> Result<String, crate::error::Error> {
    match params.first() {
        Some(Value::String(topic)) => Ok(topic.clone()),
        Some(other) => Err(crate::error::Error::InvalidInput(format!(
            "ledger.subscribe topic must be a string, got {other}"
        ))),
        None => Err(crate::error::Error::InvalidInput(
            "ledger.subscribe requires a topic parameter".to_string(),
        )),
    }
}

fn subscription_id_value_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(id) => Some(id),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn subscription_id_from_kind(kind: &SubscriptionKind) -> Result<String, crate::error::Error> {
    match kind {
        SubscriptionKind::Subscription(id) => Ok(subscription_id_to_string(id)),
        SubscriptionKind::Method(_) => Err(crate::error::Error::InvalidInput(
            "ledger.subscribe did not return a subscription id".to_string(),
        )),
        _ => Err(crate::error::Error::InvalidInput(
            "ledger.subscribe returned an unsupported subscription kind".to_string(),
        )),
    }
}

fn subscription_id_to_string(id: &SubscriptionId<'_>) -> String {
    match id {
        SubscriptionId::Num(id) => id.to_string(),
        SubscriptionId::Str(id) => id.to_string(),
    }
}

impl Client for WsClient {
    async fn send_request(&self, method: &str, params: &[Value]) -> Result<Value, ClientError> {
        let client = self.client.as_ref().ok_or(ClientError::NoConnection)?;
        if self.state != ConnectionState::Running {
            return Err(ClientError::NoConnection);
        }
        let mut rpc_params = ArrayParams::new();
        for value in params {
            rpc_params
                .insert(value.clone())
                .map_err(|_| ClientError::NoConnection)?;
        }
        client
            .request::<Value, _>(method, rpc_params)
            .await
            .map_err(|_| ClientError::NoConnection)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn connection_state_variants_are_distinct() {
        let variants = [
            ConnectionState::Uninitialized,
            ConnectionState::Connecting,
            ConnectionState::Running,
            ConnectionState::Stopped,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "distinct state variants must not be equal");
                }
            }
        }
    }

    #[tokio::test]
    async fn a_fresh_client_is_uninitialized_and_closed() {
        let client = WsClient::new();
        assert_eq!(client.status(), ConnectionState::Uninitialized);
        assert!(client.is_closed(), "a fresh client must be closed");
    }

    #[tokio::test]
    async fn send_request_fails_before_initialize() {
        let client = WsClient::new();
        let result = client.send_request("any.method", &[]).await;
        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "send_request must fail with NoConnection before initialize"
        );
    }

    #[tokio::test]
    async fn send_request_round_trips_through_a_mock_server() {
        use jsonrpsee::server::{RpcModule, ServerBuilder};

        let server = ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .expect("mock server binds a random port");
        let addr = server.local_addr().expect("server reports its address");
        let mut module: RpcModule<()> = RpcModule::new(());
        module
            .register_method("ledger.getNetworkInfo", |_params, _context, _extensions| {
                let response: jsonrpsee::core::RpcResult<serde_json::Value> =
                    Ok(serde_json::json!({ "ok": true }));
                response
            })
            .expect("mock method registers");
        let handle = server.start(module);

        let mut client = WsClient::new();
        let url = format!("ws://{addr}");
        let connected = client
            .initialize(&url, false)
            .await
            .expect("initialize connects to the mock server");
        assert!(connected, "initialize must report a successful connection");
        assert_eq!(client.status(), ConnectionState::Running);
        assert!(!client.is_closed(), "a connected client is not closed");

        let result = client
            .send_request("ledger.getNetworkInfo", &[])
            .await
            .expect("send_request round-trips the mock response");
        assert_eq!(result, serde_json::json!({ "ok": true }));

        handle.stop().expect("mock server stops");
    }

    #[tokio::test]
    async fn initialize_with_retry_makes_multiple_attempts() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};

        // Accept and drop each connection so the WebSocket handshake fails and
        // `initialize` retries. Each accept records one dial attempt.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds a free port");
        let addr = listener.local_addr().expect("reports its address");
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let server = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::Relaxed);
                drop(stream);
            }
        });

        let mut client = WsClient::new();
        let result = client.initialize(&format!("ws://{addr}"), true).await;
        server.abort();

        assert!(
            matches!(result, Err(ClientError::NoConnection)),
            "a retried dial that never connects must fail with NoConnection, got {result:?}"
        );
        assert!(
            attempts.load(Ordering::Relaxed) > 1,
            "retry=true must make more than one dial attempt, saw {}",
            attempts.load(Ordering::Relaxed)
        );
        assert!(client.is_closed(), "a failed dial leaves the client closed");
    }
}
