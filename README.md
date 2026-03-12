# Cortex

Cortex is a local-first code knowledge engine for AI agents and developer tools.

It indexes a repository into a persistent semantic graph so tools can ask structural questions like:

- what defines this symbol?
- what calls this function?
- what depends on this type?
- what might break if I change this file?

Instead of treating code as raw text, Cortex exposes machine-readable structure through a CLI and a local HTTP daemon.

## What Cortex Does

- indexes Rust, JavaScript, TypeScript, Python, and Go repositories
- persists a local graph in `sled`
- serves typed queries for symbols, dependencies, callers, callees, references, impact, and explain summaries
- supports local re-indexing and refresh
- ships as a Rust monorepo with:
  - `cortex`: CLI
  - `cortexd`: local daemon

## Status

Cortex is usable today for local structural awareness, agent context building, and pre-refactor analysis.

Current maturity:

- public OSS beta
- useful for local developer and agent workflows
- not yet compiler-grade or semantics-complete
- not yet hardened for large-scale autonomous editing across arbitrary repositories

## Why Use It

Text search is a poor interface for structural reasoning. Cortex is useful when an agent or developer needs repository context before changing code.

Examples from this repository:

- `find-symbol RepositorySession` resolves the actual owner of a core type
- `callers open_session` surfaces the CLI entrypoints that depend on it
- `dependencies RepositorySession --direction both --depth 1` exposes the nearby structural neighborhood

Measured self-host results on this repository:

- files indexed: `9`
- symbols indexed: `165`
- edges indexed: `1717`
- `RepositorySession` explain summary: `29` incoming edges, `9` outgoing edges

Additional field tests run during this release pass:

- `tokio-rs/mini-redis`: `27` files, `249` symbols, `2254` edges; `Connection` resolved to `src/connection.rs:21`, and `read_frame` surfaced `2` concrete callers
- `psf/requests`: `36` files, `759` symbols, `5373` edges; `Session` resolved to `src/requests/sessions.py:357`, with `58` impacted nodes at depth `1`
- `go-chi/chi`: `74` files, `433` symbols, `5809` edges; `NewRouter` resolved to `chi.go:60`, with `84` impacted nodes at depth `1`
- `axios/axios`: `193` files, `2985` symbols, `26340` edges; `dispatchRequest` resolved to `lib/core/dispatchRequest.js:34`, with impact pointing back to `Axios::_request`

## Installation

### Install a release binary

Unix:

```bash
curl -fsSL https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.sh | sh
```

PowerShell:

```powershell
iwr https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.ps1 -useb | iex
```

By default, the installer places binaries in:

- Unix: `~/.local/bin`
- Windows: `$HOME\.cortex\bin`

### Install from source

```bash
cargo install --path crates/cortex-cli
cargo install --path crates/cortex-daemon
```

On Windows, prefer the MSVC toolchain if the GNU toolchain does not provide `gcc.exe` and `dlltool.exe`.

## Quick Start

Index a repository:

```bash
cortex index --repo /path/to/repo
```

Find a symbol:

```bash
cortex query --repo /path/to/repo find-symbol --name RepositorySession
```

Check callers before a change:

```bash
cortex query --repo /path/to/repo callers --target open_session
```

Inspect dependencies:

```bash
cortex query --repo /path/to/repo dependencies --target RepositorySession --direction both --depth 1
```

Run the daemon:

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
```

## Agent Skill

`SKILL.md` is the agent-facing usage guide for applying Cortex to an arbitrary repository.

Use it when an agent needs to:

- resolve the real owner of a symbol
- inspect callers before changing behavior
- trace local dependency neighborhood
- estimate conservative blast radius before editing

`AGENTS.md` is different: it contains contributor instructions specific to the Cortex repository itself.

## HTTP API

Daemon endpoints:

- `POST /index/open`
- `POST /index/refresh`
- `GET /graph/find_symbol`
- `GET /graph/dependencies`
- `GET /graph/callers`
- `GET /graph/callees`
- `GET /graph/references`
- `GET /graph/impact`
- `GET /graph/explain`

Example:

```bash
curl "http://127.0.0.1:8787/graph/find_symbol?name=RepositorySession"
```

## Repository Layout

- `crates/cortex-core`: graph model, indexer, extractors, storage, and query engine
- `crates/cortex-cli`: CLI entrypoint and query commands
- `crates/cortex-daemon`: local HTTP server for tool integrations
- `site/`: Zola landing page and docs

## Development

Format, lint, and test:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Windows self-test:

```powershell
./scripts/self-test.ps1
```

## Limitations

- analysis is intentionally conservative and syntax-driven
- import and reference resolution are best-effort, not compiler-grade
- the embedded store is single-writer; use `--store-path` for isolated concurrent runs
- current strength is structural context, not full semantic truth

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md).

## License

MIT. See [`LICENSE`](./LICENSE).
