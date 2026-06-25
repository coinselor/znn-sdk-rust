# Testing with mocks

SDK and application tests do not need a funded testnet wallet. Use a local JSON-RPC mock server and assert the request/response contract.

The SDK integration tests use `jsonrpsee` to spawn an in-process WebSocket server. `tests/zenon_facade_test.rs` exercises `Zenon::send` without funds:

1. mock `ledger.getFrontierAccountBlock`
2. mock `ledger.getFrontierMomentum`
3. mock `embedded.plasma.getRequiredPoWForAccountBlock`
4. capture `ledger.publishRawTransaction`
5. parse the published template and assert address, height, momentum acknowledgement, plasma/difficulty/nonce, hash, and signature

## CI behavior

- deterministic runs
- no testnet faucet dependency
- no private keys in CI secrets
- exact JSON-RPC method and params assertions
- signed template inspection before node submission

## Pattern

```rust,no_run
use serde_json::{json, Value};
use znn_sdk_rust::zenon::Zenon;

// Pseudocode: see tests/support/mod.rs and tests/zenon_facade_test.rs for a complete harness.
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let node = spawn_mock_node(|module| {
    capture_calls(module, "ledger.getFrontierAccountBlock", Value::Null);
    capture_calls(module, "ledger.getFrontierMomentum", frontier_momentum_json());
    capture_calls(
        module,
        "embedded.plasma.getRequiredPoWForAccountBlock",
        json!({"availablePlasma": 0, "basePlasma": 21000, "requiredDifficulty": 0}),
    );
    capture_calls(module, "ledger.publishRawTransaction", Value::Null);
}).await;

let zenon = Zenon::from_client(connect(&node.url).await);
// build template + keypair, then zenon.send(...)
# Ok(())
# }
# async fn spawn_mock_node<F>(_f: F) -> MockNode where F: FnOnce(()) { todo!() }
# async fn connect(_url: &str) {}
# fn capture_calls(_: (), _: &str, _: serde_json::Value) {}
# fn frontier_momentum_json() -> serde_json::Value { serde_json::Value::Null }
# struct MockNode { url: String }
```

Use this pattern for application-level tests. Reserve live testnet tests for manual smoke checks.
