# Tests

Integration: `tests/<src_module_path>_test.rs` at this directory root only (`src/a/b.rs` → `tests/a_b_test.rs`).

Unit: `#[cfg(test)]` in `src/`.

Data: [`vectors/`](vectors/) · [`conformance/`](conformance/) · [`conformance/golden/`](conformance/golden/)

```bash
cargo test --locked
```
