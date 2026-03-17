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
- first-party extractors for Rust, JavaScript/TypeScript, Python, Go, Java, Kotlin, C#, C/C++, Swift, Objective-C, Ruby, PHP, Scala, Elixir, Erlang, Dart, Lua, R, Julia, Haskell, OCaml, Clojure, Bash, HTML, CSS, and YAML
- unsupported extensions are skipped, and coverage for non-first-class languages is less complete and precise than first-party extractors
- typed queries for `find-symbol`, `dependencies`, `callers`, `callees`, `references`, `impact`, and `explain`
- a public agent-facing [`SKILL.md`](./SKILL.md)

## Benchmark results

Latest benchmark summary (26 repos, 27 scenarios): 9087 files, 63161 symbols, 727716 edges. See [benchmarks/latest.md](./benchmarks/latest.md) and [benchmarks/latest.json](./benchmarks/latest.json).

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

## Storage compaction guarantee

Cortex keeps a single live index per repository. Each index/refresh deterministically replaces the on-disk store atomically and removes prior store blobs, so no versioned index blobs are retained.

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
