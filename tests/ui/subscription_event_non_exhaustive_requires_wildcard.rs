//! Compile-fail guard: `SubscriptionEvent` is `#[non_exhaustive]`, so a match
//! that lists every known variant but omits the `_` wildcard must fail to
//! compile. This proves the enum cannot be matched exhaustively across the crate
//! boundary, forcing callers to remain forward-compatible with future variants.

use znn_sdk_rust::api::subscribe::SubscriptionEvent;

fn main() {
    let event: SubscriptionEvent = make_event();
    // Deliberately exhaustive without a `_` arm. Because `SubscriptionEvent` is
    // `#[non_exhaustive]` and originates from another crate, the compiler must
    // reject this match.
    let _kind = match event {
        SubscriptionEvent::Momentum(_) => "momentum",
        SubscriptionEvent::AccountBlocks(_) => "account-blocks",
        SubscriptionEvent::UnreceivedAccountBlocks(_) => "unreceived",
        SubscriptionEvent::Unknown { .. } => "unknown",
    };
    let _ = _kind;
}

fn make_event() -> SubscriptionEvent {
    SubscriptionEvent::Unknown {
        topic: String::new(),
        payload: serde_json::Value::Null,
    }
}
