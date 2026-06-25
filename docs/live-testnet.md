# Live testnet checks

Normal tests use mocks and do not require funds. Live checks are ignored by default. Run them against testnet only.

## Read-only smoke test

Set a testnet WebSocket URL and run the ignored live tests:

```bash
export ZNN_TESTNET_NODE_URL='ws://127.0.0.1:35998'
cargo test --test live_testnet_test -- --ignored --nocapture
```

The read-only test connects and fetches the frontier momentum. It does not need a wallet.

## Optional live send-to-self test

This test publishes a real account block. It requires a funded testnet mnemonic and sends one smallest ZNN unit to the account's own address.

```bash
export ZNN_TESTNET_NODE_URL='ws://127.0.0.1:35998'
export ZNN_TESTNET_MNEMONIC='... testnet mnemonic ...'
export ZNN_TESTNET_SEND=1
cargo test --test live_testnet_test live_send_self_on_testnet -- --ignored --nocapture
```

Constraints:

- Use testnet only. Never put a mainnet mnemonic in CI or shell history.
- The account must have enough testnet ZNN and plasma, or must be able to generate required PoW.
- `Zenon::send` fills frontier, momentum, and plasma fields. It generates PoW when the plasma API returns a non-zero difficulty, signs the block, and calls `ledger.publishRawTransaction`.
- Node acceptance depends on protocol rules and account state. A valid local signature does not guarantee node acceptance.

## Privileged operations

Many embedded contract builders create syntactically valid templates that ordinary accounts cannot publish. Examples include spork activation and bridge administration. See [`privileged-operations.md`](privileged-operations.md).
