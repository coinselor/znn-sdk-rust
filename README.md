<div align="center">

<img src="./.github/znn-sdk-rust.png" alt="znn_sdk_rust" width="200">

# znn-sdk-rust

Rust SDK for Zenon applications, wallets, and tools.

</div>

<div align="center">
  <a href="https://crates.io/crates/znn-sdk-rust"><img src="https://img.shields.io/crates/v/znn-sdk-rust?style=for-the-badge&logo=rust&logoColor=white&color=000000" alt="crates.io"></a>
  <a href="https://docs.rs/znn-sdk-rust"><img src="https://img.shields.io/badge/docs.rs-znn--sdk--rust-000000?style=for-the-badge&logo=docs.rs&logoColor=white" alt="docs.rs"></a>
  <img src="https://img.shields.io/badge/Rust-%3E%3D1.96-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust >=1.96">
  <img src="https://img.shields.io/badge/License-MIT-000000?style=for-the-badge&logo=opensourceinitiative&logoColor=white" alt="License: MIT">
</div>

## Why

`znn_sdk_rust` provides Rust bindings for Zenon, the Network of Momentum.

The crate connects to Zenon nodes, reads JSON-RPC APIs, builds account-block
templates, derives wallet keys, signs transactions, and runs proof-of-work when
publishing requires it.

[Examples](./examples) | [Mock testing](./docs/testing-with-mocks.md) | [Live testnet](./docs/live-testnet.md) | [Privileged operations](./docs/privileged-operations.md)

## Usage

### Requirements

- Rust 1.96 or newer
- A Zenon node WebSocket endpoint for live API calls

### Install

```toml
[dependencies]
znn-sdk-rust = "0.1.0-alpha.1"
```

Or with `cargo add`:

```bash
cargo add znn-sdk-rust
```

Default features enable the WebSocket client, filesystem wallet helpers, and
native proof-of-work.

Use the reduced core without default features:

```toml
znn-sdk-rust = { version = "0.1.0-alpha.1", default-features = false }
```

The reduced core includes primitives, models, ABI helpers, deterministic crypto
helpers, embedded builders, block hashing, and proof-of-work verification
helpers.

### Connect to a node

```rust,no_run
use znn_sdk_rust::Zenon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zenon = Zenon::connect("ws://127.0.0.1:35998", false).await?;
    let frontier = zenon.ledger.get_frontier_momentum().await?;

    println!("frontier height={}", frontier.height());
    Ok(())
}
```

See [`examples/api.rs`](./examples/api.rs).

### Derive a wallet account

```rust
use znn_sdk_rust::KeyStore;

let mnemonic = "route become dream access impulse price inform obtain engage ski believe awful absent pig thing vibrant possible exotic flee pepper marble rural fire fancy";
let store = KeyStore::from_mnemonic(mnemonic)?;
let key_pair = store.get_key_pair(0)?;

println!("address={}", key_pair.address()?);
# Ok::<(), znn_sdk_rust::Error>(())
```

See [`examples/wallet.rs`](./examples/wallet.rs).

### Send a transaction

`Zenon::send` fills account-block fields from the connected node, checks whether
PoW is required, signs the transaction hash, and publishes the signed template.

```rust,no_run
use num_bigint::BigUint;
use znn_sdk_rust::{AccountBlockTemplate, Address, KeyStore, Zenon};
use znn_sdk_rust::primitives::token_standard::znn_token_standard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zenon = Zenon::connect("ws://127.0.0.1:35998", false).await?;
    let store = KeyStore::from_mnemonic(std::env::var("ZNN_MNEMONIC")?.as_str())?;
    let key_pair = store.get_key_pair(0)?;

    let to = Address::parse("z1qr4pexnnfaexqqz8nscjjcsajy5hdqfkgadvwx")?;
    let template = AccountBlockTemplate::send(to, znn_token_standard(), BigUint::from(1u32), None);
    let response = zenon.send(&template, Some(&key_pair)).await?;

    println!("publish response={response}");
    Ok(())
}
```

Use testnet for live send checks. See [`examples/send.rs`](./examples/send.rs)
and [`docs/live-testnet.md`](./docs/live-testnet.md). Never use a mainnet
mnemonic in CI or shell history.

## API overview

Most applications can import what they need directly from the crate root: the
`Zenon` entry point, the `Error` type, the JSON-RPC API roots (`LedgerApi`,
`StatsApi`, `SubscribeApi`, `EmbeddedApi`), core primitives (`Address`, `Hash`,
`HashHeight`, `TokenStandard`), the NOM `AccountBlockTemplate` and `BlockType`,
and wallet key types (`KeyPair`, `KeyStore`).

Lower-level modules remain available for specialized use: `abi`, `embedded`,
`model`, `utils`, `pow`, `wallet`, and `client`.

## Development

```bash
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --no-deps
```

Live tests are ignored by default and require explicit testnet environment
variables. Normal integration tests use local JSON-RPC mocks and do not require
funds.

## Contributing

Issues and pull requests are welcome. Add tests for behavior changes. Run the
commands above. Call out changes to `Cargo.lock` or `deny.toml` for review.

## License

[MIT](./Cargo.toml)
