# Contributing

## Prerequisites

- Rust stable
- `clippy` and `rustfmt`
- On Windows, prefer the MSVC toolchain

## Development

```bash
cargo +stable-x86_64-pc-windows-msvc fmt
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test
```

## Project Layout

- `crates/cortex-core`: indexing, storage, graph model, queries
- `crates/cortex-cli`: local CLI
- `crates/cortex-daemon`: local HTTP daemon

## Pull Requests

- Keep changes scoped and documented
- Add tests for behavior changes
- Update the README when CLI, daemon, or install behavior changes
