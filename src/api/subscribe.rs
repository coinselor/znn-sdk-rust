//! Subscribe API (`ledger.subscribe` JSON-RPC methods).
//!
//! [`SubscribeApi`] wraps a shared [`Client`] and dispatches the
//! `ledger.subscribe` method for momentum and account-block subscription roots.

use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::error::Error;
use crate::primitives::address::Address;
use jsonrpsee::core::client::Subscription as RpcSubscription;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

/// A handle delivering `ledger.subscription` notifications for a single
/// subscription id.
///
/// The handle owns the jsonrpsee subscription stream plus a fallback receiver
/// for legacy method-notification payloads. When the client is stopped or the
/// connection closes, [`Subscription::recv`] finishes with `Ok(None)`. Handles
/// closed by `WsClient::stop` or a reconnect are not resubscribed; create a new
/// stream handle after reconnecting.
pub struct Subscription {
    id: String,
    topic: String,
    receiver: Option<mpsc::UnboundedReceiver<Value>>,
    rpc: Option<RpcSubscription<Value>>,
    registry_cleanup: Option<Box<dyn FnOnce() + Send + 'static>>,
    client_stopped: Arc<AtomicBool>,
    closed: bool,
}

impl Subscription {
    /// Creates a handle scoped to `id` reading from the jsonrpsee subscription
    /// stream and a fallback SDK-owned notification channel. `client_stopped`
    /// is shared with the owning [`WsClient`] so a closure can be classified as
    /// [`CloseReason::ClientStopped`] versus a transport reconnect.
    pub(crate) fn from_rpc(
        id: String,
        topic: String,
        rpc: RpcSubscription<Value>,
        receiver: mpsc::UnboundedReceiver<Value>,
        registry_cleanup: Option<Box<dyn FnOnce() + Send + 'static>>,
        client_stopped: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            topic,
            receiver: Some(receiver),
            rpc: Some(rpc),
            registry_cleanup,
            client_stopped,
            closed: false,
        }
    }

    /// Returns the subscription id this handle receives notifications for.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Awaits the next notification payload for this subscription.
    ///
    /// Returns `Ok(Some(payload))` when a notification arrives, `Ok(None)` when
    /// the subscription has closed gracefully (the client was stopped, the
    /// connection closed, or the subscription was dropped), or [`Error`] when
    /// the client surfaces a malformed notification payload.
    pub async fn recv(&mut self) -> Result<Option<Value>, Error> {
        match (&mut self.rpc, &mut self.receiver) {
            (Some(rpc), Some(receiver)) => {
                tokio::select! {
                    next = rpc.next() => rpc_next_to_result(next),
                    next = receiver.recv() => Ok(next),
                }
            }
            (Some(rpc), None) => rpc_next_to_result(rpc.next().await),
            (None, Some(receiver)) => Ok(receiver.recv().await),
            (None, None) => Ok(None),
        }
    }

    /// Awaits the next notification or a classified closure event.
    ///
    /// Like [`Self::recv`] but reports *why* the handle closed via
    /// [`SubscriptionOutcome::Closed`] with a [`CloseReason`]. After a `Closed`
    /// outcome, subsequent calls return `Ok(None)`.
    ///
    /// The closure cause is classified from the shared `client_stopped` flag: a
    /// stop set by [`WsClient::stop`] yields [`CloseReason::ClientStopped`];
    /// any other end of stream (transport drop or server-initiated close while
    /// the client stays `Running`) yields [`CloseReason::TransportReconnected`].
    pub async fn recv_event(&mut self) -> Result<Option<SubscriptionOutcome>, Error> {
        if self.closed {
            return Ok(None);
        }
        if let Some(value) = self.recv().await? {
            return Ok(Some(SubscriptionOutcome::Item(value)));
        }
        self.closed = true;
        let reason = if self.client_stopped.load(Ordering::Acquire) {
            CloseReason::ClientStopped
        } else {
            CloseReason::TransportReconnected
        };
        Ok(Some(SubscriptionOutcome::Closed { reason }))
    }

    /// Awaits the next notification and decodes it into a typed
    /// [`SubscriptionEvent`].
    ///
    /// Decode failures are item-level: a malformed payload returns
    /// [`Error::InvalidInput`] for that call and the handle stays usable for the
    /// next notification.
    ///
    /// Reads the next raw notification via [`Self::recv`] and routes it through
    /// [`SubscriptionEvent::decode`] keyed on this handle's subscribe topic:
    /// known topics decode to their typed variant, and an unmodeled topic falls
    /// back to [`SubscriptionEvent::Unknown`] carrying the raw payload.
    pub async fn recv_typed(&mut self) -> Result<Option<SubscriptionEvent>, Error> {
        match self.recv().await? {
            None => Ok(None),
            Some(value) => Ok(Some(SubscriptionEvent::decode(self.topic(), &value)?)),
        }
    }

    /// The subscribe topic this handle was opened for.
    fn topic(&self) -> &str {
        &self.topic
    }

    /// Non-blocking check for a pending notification payload.
    ///
    /// Returns `Ok(Some(payload))` when a fallback notification is already
    /// queued, `Ok(None)` when none is pending or the subscription has closed.
    /// Standard jsonrpsee subscription streams are polled by [`Self::recv`].
    pub fn try_recv(&mut self) -> Result<Option<Value>, Error> {
        let Some(receiver) = self.receiver.as_mut() else {
            return Ok(None);
        };
        match receiver.try_recv() {
            Ok(value) => Ok(Some(value)),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                Ok(None)
            }
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(cleanup) = self.registry_cleanup.take() {
            cleanup();
        }
    }
}

fn rpc_next_to_result(
    next: Option<Result<Value, serde_json::Error>>,
) -> Result<Option<Value>, Error> {
    match next {
        Some(Ok(value)) => Ok(Some(value)),
        Some(Err(err)) => Err(Error::Serialization(err.to_string())),
        None => Ok(None),
    }
}

/// Why a subscription handle stopped delivering notifications.
///
/// Surfaced through [`Subscription::recv_event`] so callers can distinguish a
/// graceful client stop from a transport reconnect or a server-initiated close.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// `WsClient::stop` was called.
    ClientStopped,
    /// The websocket transport dropped and was re-established; the server-side
    /// subscription id is no longer valid.
    TransportReconnected,
    /// The server ended the subscription while the client stayed connected.
    ///
    /// On jsonrpsee 0.26 and the current wire protocol, a server-initiated
    /// close produces no client-observable signal. `next()` stops resolving and
    /// `close_reason()` stays `None` while the transport stays open. Such a
    /// close is reported as `TransportReconnected` once the stream is observed
    /// to end. This variant remains for a future protocol/transport revision
    /// that distinguishes the two causes.
    ServerInitiated,
}

/// Outcome of a [`Subscription::recv_event`] poll.
///
/// Carries a raw notification item, or a one-time `Closed` marker with the
/// classified [`CloseReason`]. After `Closed` is returned, subsequent
/// `recv_event` calls return `Ok(None)`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionOutcome {
    /// A raw notification payload arrived.
    Item(Value),
    /// The handle is inert and will deliver no more items.
    Closed {
        /// Why the handle closed.
        reason: CloseReason,
    },
}

/// Opt-in automatic resubscribe for transport reconnects.
///
/// Holds the subscribe intent (method + params) and the connection lifecycle.
/// After a transport drop and a [`Self::reconnect`], [`Self::resubscribe_if_needed`]
/// re-issues `ledger.subscribe` and hands back a freshly bound
/// [`Subscription`] under the new server id. The manager, not the handle, owns
/// the subscribe intent across reconnects. The caller swaps to the rebound
/// handle returned by the manager. The manager does not mutate a handle it no
/// longer owns. Default [`Subscription`] handles never auto-resubscribe; this
/// manager is the explicit opt-in layer.
///
/// The implementation records the subscribe intent (topic + params) in
/// [`Self::subscribe_momentums`] so [`Self::resubscribe_if_needed`] knows what to
/// re-issue. With no recorded intent the latter is a no-op returning `Ok(None)`;
/// with a recorded intent it returns `Ok(Some(handle))` on success or `Err` if
/// the re-subscribe RPC fails. Because the rebound subscription carries a new
/// server id and the old transport's registry was cleared on the disconnect (and
/// the old handle deregisters its id on `Drop`), the old and new ids never both
/// deliver. This avoids duplicate delivery without shared mutable handle state.
pub struct ResubscribeManager {
    client: WsClient,
    subscribe_intent: Option<(String, Vec<Value>)>,
}

impl ResubscribeManager {
    /// Creates a manager that owns `client` and its connection lifecycle.
    pub fn new(client: WsClient) -> Self {
        Self {
            client,
            subscribe_intent: None,
        }
    }

    /// Subscribes to new momentums through the manager, recording the subscribe
    /// intent so [`Self::resubscribe_if_needed`] can re-issue it after a
    /// reconnect.
    pub async fn subscribe_momentums(&mut self) -> Result<Subscription, Error> {
        let params = vec![json!("momentums")];
        let handle = self.client.subscribe_stream(&params).await?;
        self.subscribe_intent = Some(("momentums".to_string(), params));
        Ok(handle)
    }

    /// Re-dials `url` after a transport drop, restoring the live transport.
    pub async fn reconnect(&mut self, url: &str) -> Result<(), Error> {
        self.client
            .initialize(url, true)
            .await
            .map(|_| ())
            .map_err(crate::error::Error::from)
    }

    /// Re-issues the recorded subscription after a transport reconnect and
    /// returns a freshly bound handle.
    ///
    /// Returns `Ok(Some(handle))` carrying a new server id when a prior intent
    /// was recorded and the re-subscribe RPC succeeded; `Ok(None)` when there is
    /// nothing to resubscribe (no prior intent); and `Err` when the re-subscribe
    /// RPC fails (the caller is never silently left without a working handle).
    pub async fn resubscribe_if_needed(&mut self) -> Result<Option<Subscription>, Error> {
        let Some((_topic, params)) = &self.subscribe_intent else {
            return Ok(None);
        };
        let handle = self.client.subscribe_stream(params).await?;
        Ok(Some(handle))
    }
}

/// A typed `ledger.subscription` notification.
///
/// Known topics decode to the matching variant; unmodeled topics fall back to
/// [`SubscriptionEvent::Unknown`] carrying the raw payload so a newer node plus
/// an older SDK never loses data silently. The enum is `#[non_exhaustive]`, so
/// callers must include a `_` arm when matching.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SubscriptionEvent {
    /// New-momentum notification for a `momentums` subscription.
    Momentum(Box<crate::model::nom::momentum::Momentum>),
    /// Account-block notification for `allAccountBlocks` /
    /// `accountBlocksByAddress`.
    AccountBlocks(Vec<crate::model::nom::account_block::AccountBlock>),
    /// Unreceived account-block notification for
    /// `unreceivedAccountBlocksByAddress`.
    UnreceivedAccountBlocks(Vec<crate::model::nom::account_block::AccountBlock>),
    /// A notification whose topic is not modeled by this SDK version.
    Unknown {
        /// The subscribe topic string.
        topic: String,
        /// The raw decoded payload.
        payload: Value,
    },
}

impl SubscriptionEvent {
    /// Decodes a raw notification `payload` routed under `topic` into a typed
    /// event. A malformed payload returns [`Error::InvalidInput`]; an unmodeled
    /// topic returns [`SubscriptionEvent::Unknown`].
    pub fn decode(topic: &str, payload: &Value) -> Result<Self, Error> {
        match topic {
            "momentums" => {
                let momentum = crate::model::nom::momentum::Momentum::from_json(payload)?;
                Ok(Self::Momentum(Box::new(momentum)))
            }
            "allAccountBlocks" | "accountBlocksByAddress" => {
                let blocks = parse_account_block_array(payload)?;
                Ok(Self::AccountBlocks(blocks))
            }
            "unreceivedAccountBlocksByAddress" => {
                let blocks = parse_account_block_array(payload)?;
                Ok(Self::UnreceivedAccountBlocks(blocks))
            }
            _ => Ok(Self::Unknown {
                topic: topic.to_string(),
                payload: payload.clone(),
            }),
        }
    }
}

fn parse_account_block_array(
    payload: &Value,
) -> Result<Vec<crate::model::nom::account_block::AccountBlock>, Error> {
    let array = payload.as_array().ok_or_else(|| {
        Error::InvalidInput(format!(
            "account-block notification must be an array, got {payload}"
        ))
    })?;
    array
        .iter()
        .map(crate::model::nom::account_block::AccountBlock::from_json)
        .collect()
}

/// The `ledger.subscribe` JSON-RPC namespace.
pub struct SubscribeApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> SubscribeApi<C> {
    /// Creates a subscribe API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Subscribes to new momentums.
    pub async fn to_momentums(&self) -> Result<Option<String>, Error> {
        self.subscribe(&[json!("momentums")]).await
    }

    /// Subscribes to all account blocks.
    pub async fn to_all_account_blocks(&self) -> Result<Option<String>, Error> {
        self.subscribe(&[json!("allAccountBlocks")]).await
    }

    /// Subscribes to account blocks for `address`.
    pub async fn to_account_blocks_by_address(
        &self,
        address: &Address,
    ) -> Result<Option<String>, Error> {
        self.subscribe(&[json!("accountBlocksByAddress"), json!(address.to_string())])
            .await
    }

    /// Subscribes to unreceived account blocks for `address`.
    pub async fn to_unreceived_account_blocks_by_address(
        &self,
        address: &Address,
    ) -> Result<Option<String>, Error> {
        self.subscribe(&[
            json!("unreceivedAccountBlocksByAddress"),
            json!(address.to_string()),
        ])
        .await
    }

    async fn subscribe(&self, params: &[Value]) -> Result<Option<String>, Error> {
        let response = self
            .client
            .send_request("ledger.subscribe", params)
            .await
            .map_err(Error::from)?;
        match response {
            Value::Null => Ok(None),
            Value::String(id) => Ok(Some(id)),
            other => Err(Error::InvalidInput(format!(
                "ledger.subscribe response must be a string or null, got {other}"
            ))),
        }
    }
}

impl SubscribeApi<WsClient> {
    /// Subscribes to new momentums and returns a notification handle.
    ///
    /// Handles closed by `WsClient::stop` or a reconnect stay closed; call this
    /// method again after reconnecting to receive new notifications.
    pub async fn to_momentums_stream(&self) -> Result<Subscription, Error> {
        self.client.subscribe_stream(&[json!("momentums")]).await
    }

    /// Subscribes to all account blocks and returns a notification handle.
    pub async fn to_all_account_blocks_stream(&self) -> Result<Subscription, Error> {
        self.client
            .subscribe_stream(&[json!("allAccountBlocks")])
            .await
    }

    /// Subscribes to account blocks for `address` and returns a handle.
    pub async fn to_account_blocks_by_address_stream(
        &self,
        address: &Address,
    ) -> Result<Subscription, Error> {
        self.client
            .subscribe_stream(&[json!("accountBlocksByAddress"), json!(address.to_string())])
            .await
    }

    /// Subscribes to unreceived account blocks for `address` and returns a handle.
    pub async fn to_unreceived_account_blocks_by_address_stream(
        &self,
        address: &Address,
    ) -> Result<Subscription, Error> {
        self.client
            .subscribe_stream(&[
                json!("unreceivedAccountBlocksByAddress"),
                json!(address.to_string()),
            ])
            .await
    }
}
