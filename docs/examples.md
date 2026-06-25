# Examples

Runnable examples: [`../examples`](../examples)

- [`api.rs`](../examples/api.rs): connect to a node and read the frontier momentum.
- [`wallet.rs`](../examples/wallet.rs): derive account `0` from a mnemonic.
- [`send.rs`](../examples/send.rs): build and publish a ZNN send template.
- [`embedded.rs`](../examples/embedded.rs): build stake, plasma fuse, and token issue templates.

```bash
cargo run --example wallet
ZNN_NODE_URL='ws://127.0.0.1:35998' cargo run --example api
```

For examples that publish blocks, use testnet and read [`live-testnet.md`](live-testnet.md) first.
