# Cortex

Cortex is a local-first code knowledge engine for coding agents and developer tools.

It indexes a repository into a persistent semantic graph so tools can ask structural questions like:

- what defines this symbol?
- what calls this function?
- what depends on this type?
- what is the likely blast radius of a change?

The goal is not “AI code search.” The goal is giving agents a reusable structural memory layer instead of forcing every task through raw text search and guesswork.

## Why use it

Text search is a poor interface for structural reasoning. Grep can tell you that a name appears in many places; it cannot tell you which definition is canonical, which callers are relevant, or which neighbors form the local dependency neighborhood.

Cortex is useful when an agent or developer needs:

- symbol ownership before editing an unfamiliar codebase
- callers and dependencies before a refactor
- a bounded impact estimate before changing behavior
- machine-readable structure for local tools and automations

## What ships today

- `cortex`: local CLI
- `cortexd`: local HTTP daemon
- `cortex-core`: Rust library with indexer, graph store, extractors, and query engine
- first-party extractors for Rust, JavaScript/TypeScript, Python, Go, Java, C#, Ruby, PHP, C, and C++
- intelligent heuristic fallback for unsupported languages
- typed queries for `find-symbol`, `dependencies`, `callers`, `callees`, `references`, `impact`, and `explain`
- a public agent-facing [`SKILL.md`](./SKILL.md)
- reproducible benchmark artifacts in [`benchmarks/latest.md`](./benchmarks/latest.md) and [`benchmarks/latest.json`](./benchmarks/latest.json)

## Benchmark snapshot

The current benchmark artifact was generated with the release binary on this machine using [`scripts/benchmark.ps1`](./scripts/benchmark.ps1). It compares Cortex against a raw text-search baseline: `git grep -n -w`.

Check [`benchmarks/latest.md`](benchmarks/latest.md) for detailed statistics across all supported languages (including Rust, JavaScript/TypeScript, Python, Go, Java, C#, Ruby, PHP, C, and C++). The benchmark measures structural query execution vs standard file text search using `git grep`.

Headline result:

- Cortex can radically reduce the structural search-space for queries like "what implements/defines X" and "who calls Y".
- Often, Cortex drops hundreds of raw grep lines across dozens of files down to exactly the single relevant node or edge, increasing coding agent success rates.

This is the right way to read the benchmark: Cortex is not trying to beat grep on “find bytes in files.” It is trying to reduce the amount of irrelevant text an agent has to inspect to answer a structural question.

## Installation

### Install a release binary

Unix:

```bash
curl -fsSL https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.sh | sh
```

PowerShell:

```powershell
irm https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.ps1 | iex
```

Default install locations:

- Unix: `~/.local/bin`
- Windows: `$HOME\.cortex\bin`

Windows release-installer behavior:

- installs `cortex.exe` and `cortexd.exe` into `$HOME\.cortex\bin` by default
- adds that directory to the current PowerShell session PATH immediately
- adds that directory to the user PATH for new shells unless `CORTEX_INSTALL_DIR` is explicitly set

If you set `CORTEX_INSTALL_DIR`, the installer will use that directory but will not persist it into the user PATH automatically.

### Install from source

```bash
git clone https://github.com/Blu3Ph4ntom/cortex.git
cd cortex
cargo install --path crates/cortex-cli
cargo install --path crates/cortex-daemon
```

On Windows, prefer the MSVC toolchain if the GNU toolchain does not provide `gcc.exe` and `dlltool.exe`.

## Quick start

Index a repository:

```bash
cortex index --repo /path/to/repo
```

Resolve a symbol owner:

```bash
cortex query --repo /path/to/repo find-symbol --name PaymentService
```

Check callers before a change:

```bash
cortex query --repo /path/to/repo callers --target update_status
```

Inspect dependencies and blast radius:

```bash
cortex query --repo /path/to/repo dependencies --target PaymentService --direction both --depth 1
cortex query --repo /path/to/repo impact --target update_status --depth 1
```

Run the daemon:

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
```

## Agent skill

Cortex ships an agent-facing [`SKILL.md`](./SKILL.md) for use on arbitrary repositories.

Install that skill into a compatible runtime:

```bash
npx skills add https://github.com/Blu3Ph4ntom/cortex --skill cortex
```

Direct skill URLs:

- GitHub: <https://github.com/Blu3Ph4ntom/cortex/blob/main/SKILL.md>
- Raw: <https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/SKILL.md>

What the skill is for:

- resolve the real owner of a symbol
- inspect callers before changing behavior
- trace the local dependency neighborhood
- estimate conservative blast radius before editing

`AGENTS.md` is different. It contains contributor instructions specific to working on the Cortex repository itself.

## Field-tested repositories

These runs were executed directly with Cortex during this release pass:

- `Cortex` self-host: `13` files, `323` symbols, `2,914` edges; `RepositorySession` resolved to `crates/cortex-core/src/indexer.rs:38`; `open_session` impact returned `4` nodes and `12` supporting edges
- `tokio-rs/mini-redis`: `27` files, `249` symbols, `2,254` edges; `Connection` resolved to `src/connection.rs:21`; `read_frame` surfaced `2` concrete callers
- `psf/requests`: `36` files, `759` symbols, `5,373` edges; `Session` resolved to `src/requests/sessions.py:357`; impact returned `58` nodes and `116` supporting edges
- `go-chi/chi`: `74` files, `433` symbols, `5,809` edges; `NewRouter` resolved to `chi.go:60`; impact returned `84` nodes and `252` supporting edges
- `axios/axios`: `193` files, `2,985` symbols, `26,340` edges; `dispatchRequest` resolved to `lib/core/dispatchRequest.js:34`; impact connected it back to `Axios::_request`

## Benchmarking Cortex yourself

Build the release binary first:

```bash
cargo build --release
```

Then run the benchmark harness:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\benchmark.ps1
```

Outputs:

- [`benchmarks/latest.md`](./benchmarks/latest.md): human-readable benchmark report
- [`benchmarks/latest.json`](./benchmarks/latest.json): machine-readable benchmark data
- [`site/data/benchmarks.json`](./site/data/benchmarks.json): website data source

Current methodology:

- cold index medians come from fresh stores
- warm query medians reuse a prepared local store
- the baseline is `git grep -n -w`
- the benchmark focuses on structural tasks, not byte-search throughput

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

## Repository layout

- `crates/cortex-core`: graph model, indexer, extractors, storage, and query engine
- `crates/cortex-cli`: CLI entrypoint and query commands
- `crates/cortex-daemon`: local HTTP server for tool integrations
- `site/`: Zola landing page and docs
- `benchmarks/`: generated benchmark artifacts

## Development

Format, lint, and test:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Self-test:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\self-test.ps1
```

## Status and limitations

Cortex is useful today for local structural awareness and agent workflows. It is still OSS beta software.

Current limits:

- analysis is conservative and syntax-driven
- import and reference resolution are best-effort, not compiler-grade
- runtime data flow and dynamic dispatch still need manual confirmation
- the embedded store is single-writer; use `--store-path` for isolated concurrent runs
- current strength is structural context, not full semantic truth

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md).

## License

MIT. See [`LICENSE`](./LICENSE).
