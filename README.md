# Cortex

Cortex is a local-first code intelligence engine for AI agents and developer tools. It turns a repository into a persistent semantic graph so tools can ask structural questions about symbols, files, call relationships, and impact instead of guessing with text search.

Website:

- docs and landing: `https://blu3ph4ntom.github.io/cortex/`
- GitHub Pages custom domain configured: `http://cortex.bluephantom.dev/`

## Why It Is Useful

When an agent or developer asks questions like:

- what calls this function?
- what depends on this type?
- where should I start reading?
- what might break if I change this symbol?

plain text search usually returns too much noise or misses the structural relationship entirely. Cortex gives you machine-readable graph answers instead.

In practice, even on this small self-hosting repo, Cortex already provides three useful advantages:

- **Fast structural lookup**: `find-symbol RepositorySession` resolves the exact core type definition instead of a pile of mentions.
- **Call graph discovery**: `callers open_session` returns the actual CLI entrypoints that use it: `main`, `run_index`, `run_query`, and `run_watch`.
- **Impact and dependency context**: `dependencies RepositorySession --direction both` shows both users of the type and core internals it relies on, which is the kind of context an agent needs before editing.

## What Cortex Does Today

- Indexes Rust, TypeScript/JavaScript, Python, and Go repositories
- Stores a persistent local graph in `sled`
- Supports full re-index and incremental refresh
- Exposes typed queries for:
  - symbol lookup
  - dependencies
  - callers / callees
  - references
  - conservative impact analysis
  - explain summaries
- Ships both a CLI and a local HTTP daemon

## Measured Self-Host Results

These are real results from running Cortex against this repository itself:

- `cortex index --repo . --store-path .cortex-readme`
  - indexed files: `9`
  - indexed symbols: `165`
  - indexed edges: `1717`
- `cortex query --repo . --store-path .cortex-readme find-symbol --name RepositorySession`
  - resolved the `RepositorySession` type in `crates/cortex-core/src/indexer.rs`
- `cortex query --repo . --store-path .cortex-readme callers --target open_session`
  - found `main`, `run_index`, `run_query`, and `run_watch` in the CLI crate
- `cortex query --repo . --store-path .cortex-readme explain --target RepositorySession`
  - reported `29` incoming edges and `9` outgoing edges
- `cortex query --repo . --store-path .cortex-readme dependencies --target RepositorySession --direction both --depth 1`
  - surfaced direct consumers like CLI `open_session` and daemon `open_index`
  - surfaced core neighbors like `Indexer`, `SledGraphStore`, and `DefaultExtractorRegistry`

That means Cortex is already useful as:

- a pre-edit context builder for AI agents
- a quick architecture inspection tool
- a local dependency explorer
- a regression aid for refactors

## Good Use Cases

- Give an AI coding agent structural context for a repo before editing
- Trace which local functions call a symbol
- Estimate what might break if a function or file changes
- Inspect a repository’s symbol inventory quickly
- Build agent-side tools that need machine-readable code relationships
- Generate review context for unfamiliar codebases
- Confirm call paths before refactors or API changes

## What It Is Not Yet

- Not a compiler-grade whole-program analysis engine
- Not a runtime data-flow engine
- Not a cross-repo knowledge layer yet
- Not a hosted SaaS or multi-tenant control plane
- Not a visualization-first product today

## Installation

### Prerequisites

- Rust stable
- `rustfmt` and `clippy`
- On Windows, prefer the MSVC toolchain

### Run From Source

```bash
cargo run -p cortex-cli --bin cortex -- index --repo .
cargo run -p cortex-cli --bin cortex -- query --repo . find-symbol --name main
cargo run -p cortex-daemon --bin cortexd -- --repo . --bind 127.0.0.1:8787
```

### Install The Binaries

```bash
cargo install --path crates/cortex-cli
cargo install --path crates/cortex-daemon
```

That installs:

- `cortex`: CLI
- `cortexd`: daemon

## CLI Quick Start

Index a repo:

```bash
cortex index --repo /path/to/repo
```

Inspect health:

```bash
cortex doctor --repo /path/to/repo
```

Find a symbol:

```bash
cortex query --repo /path/to/repo find-symbol --name RepositorySession
```

Trace callers:

```bash
cortex query --repo /path/to/repo callers --target open_session
```

Run a dependency walk:

```bash
cortex query --repo /path/to/repo dependencies --target RepositorySession --direction both --depth 2
```

Export the graph:

```bash
cortex export --repo /path/to/repo
```

Use a custom store path when you want isolated runs:

```bash
cortex index --repo /path/to/repo --store-path /tmp/cortex-store
```

## Real Query Scenarios

### 1. Find the core type behind an API

```bash
cortex query --repo . --store-path .cortex-readme find-symbol --name RepositorySession
```

Why this matters:

- jumps directly to the semantic definition
- avoids sorting through imports, references, and docs mentions

### 2. Find who invokes a helper before changing it

```bash
cortex query --repo . --store-path .cortex-readme callers --target open_session
```

Observed result on this repo:

- `main`
- `run_index`
- `run_query`
- `run_watch`

Why this matters:

- tells you which flows will feel the change
- gives agents a concrete edit blast radius

### 3. Inspect the neighborhood around a core type

```bash
cortex query --repo . --store-path .cortex-readme dependencies --target RepositorySession --direction both --depth 1
```

Observed result on this repo:

- inbound users include CLI and daemon functions
- direct neighbors include `Indexer`, `SledGraphStore`, and `DefaultExtractorRegistry`

Why this matters:

- helps an implementer understand a subsystem without reading every file
- gives a realistic “what is this connected to?” answer

### 4. Get a short explanation for an agent prompt

```bash
cortex query --repo . --store-path .cortex-readme explain --target RepositorySession
```

Observed result on this repo:

- `RepositorySession` currently has `29` incoming and `9` outgoing edges

Why this matters:

- useful as compressed context for agents
- useful for ranking architectural hotspots

## Daemon API

Start the daemon:

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
```

Endpoints:

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

## Self-Tested Scenarios

The repository is self-hosting enough to be useful on its own codebase. The current self-test pass covers:

- fresh index into an isolated store path
- doctor output after indexing
- symbol search for `main`
- caller search for `open_session`
- explain report for `RepositorySession`
- daemon startup with an isolated store path
- HTTP query smoke test to `/graph/find_symbol`
- core incremental refresh regression tests

The automation used for this lives in `scripts/self-test.ps1`, and validation notes live in `docs/SELF-TEST.md`.

## Development

Format, lint, and test:

```bash
cargo fmt
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test
powershell -ExecutionPolicy Bypass -File .\scripts\self-test.ps1
```

## Repository Layout

- `crates/cortex-core`: graph model, indexing, extraction, persistence, queries
- `crates/cortex-cli`: interactive CLI for local workflows
- `crates/cortex-daemon`: local HTTP daemon for agents and IDEs

## Current Limitations

- Symbol extraction is intentionally conservative and syntax-driven
- References can be noisy in large files because they are derived from identifier usage
- Import resolution is best-effort and local-first, not full compiler resolution
- The embedded store is single-writer; use `--store-path` when you need isolated concurrent runs
- Query usefulness is strongest for local structural questions, not deep semantic truth

## License

MIT. See `LICENSE`.
