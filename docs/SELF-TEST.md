# Self-Test Notes

This document records the validation scenarios run against Cortex using this repository as the target codebase.

## Automated Scenarios

The script [`scripts/self-test.ps1`](A:/Projects/cortex/scripts/self-test.ps1) validates:

- fresh index into an isolated store path
- doctor output after indexing
- symbol search for `main`
- caller search for `open_session`
- explain report for `RepositorySession`
- daemon startup with an isolated store path
- HTTP query to `/graph/find_symbol`

## Expected Value

When Cortex is useful on its own repo, it should:

- find the CLI and daemon `main` functions correctly
- identify `open_session` callers inside the CLI
- explain `RepositorySession` as a core type in the indexer layer
- return machine-readable JSON consistently from both CLI and daemon

## Current Readiness

- The core workspace compiles cleanly under the Windows MSVC toolchain
- Core tests pass
- CLI indexing and querying work against the repository itself
- Daemon smoke tests are scripted and repeatable

## Latest Observed Run On This Repository

- `cortex index --repo . --store-path .cortex-readme`
  - indexed files: `9`
  - indexed symbols: `165`
  - indexed edges: `1717`
- `cortex query --repo . find-symbol --name RepositorySession`
  - returned the `RepositorySession` type in `crates/cortex-core/src/indexer.rs`
- `cortex query --repo . callers --target open_session`
  - returned `main`, `run_index`, `run_query`, and `run_watch` as callers in the CLI crate
- `cortex query --repo . explain --target RepositorySession`
  - reported `29` incoming edges and `9` outgoing edges

## Known Constraints

- On this machine, the GNU Rust toolchain lacks `gcc.exe` and `dlltool.exe`; use the MSVC toolchain instead
- The current graph model prefers deterministic local structure over deep compiler-level precision
