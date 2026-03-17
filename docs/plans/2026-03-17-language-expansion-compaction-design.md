# Cortex Design: Auto-Compaction and Multi-Language Expansion

Date: 2026-03-17

## Goals

- Ensure each index/refresh produces a single live store with no retained historical blobs.
- Expand first-class explicit language support to 20+ languages with framework coverage.
- Validate quality with production-grade benchmarks across representative repositories.
- Update agent guidance to always build/refresh the index when needed.

## Non-Goals

- Switching storage engines (keep sled for now).
- Adding versioned storage history.

## Architecture

### Store Replacement Compaction (Deterministic)

Replace the current “rewrite state into the same sled DB” compaction with a deterministic, atomic store replacement:

1. Build graph state in memory (as today).
2. Write the new state to a fresh store path (e.g., `.cortex/index.next`).
3. Write the JSON snapshot next to the store (`.cortex/state.json`).
4. Atomically swap directories:
   - rename `.cortex/index` → `.cortex/index.prev`
   - rename `.cortex/index.next` → `.cortex/index`
5. Remove `.cortex/index.prev`.

This guarantees only the current index store remains after each index/refresh, regardless of sled’s blob behavior.

### Language Expansion Strategy

- Keep language-level extractors as the core architecture.
- Add framework-specific extraction rules only where necessary (routing, DI, annotations/decorators).
- Maintain a benchmark corpus of real, active repos for each language/framework.

## Components

- **Storage layer (cortex-core)**: new store swap + cleanup logic, snapshot synchronization.
- **Indexer**: write to a fresh store each run; expose “bytes replaced” instead of sled compaction metrics.
- **Extractors**: expand to 20+ languages; add targeted framework rules as needed.
- **Benchmark harness**: scale `scripts/benchmark.ps1` with repo matrix; update benchmark artifacts.
- **Docs / Skill**: update `SKILL.md` to mandate index/refresh usage and optional watch mode.

## Data Flow

### Index/Refresh

1. Build documents → graph snapshot.
2. Write store to `.cortex/index.next`.
3. Write `.cortex/state.json`.
4. Swap `.cortex/index.next` into `.cortex/index`.
5. Delete prior store.

### Queries

- Prefer the JSON snapshot for read-only queries.
- Fall back to sled open if snapshot unavailable.

### Benchmarks

- Clone/pull repos into a local, gitignored benchmark workspace.
- Run index + query scenarios.
- Publish `benchmarks/latest.md` and `benchmarks/latest.json` (and website data file).

## Error Handling & Safety

- If swap fails, keep the prior store intact and return an error.
- If temp store build fails, remove temp store and keep prior store.
- Clean up `.cortex/index.next` or `.cortex/index.prev` on next startup if present.
- Maintain single-writer discipline; readers use snapshot.

## Testing & Validation

- Unit tests for swap logic and failure paths.
- Integration test: multiple index runs leave a single store directory and stable blob counts.
- Snapshot correctness regression tests.
- Extractor tests per language and framework.
- Benchmark runs across 20+ repos with reported metrics.

## Language/Framework Coverage Targets

Initial target set (20+):

- Rust, Go, Python, JavaScript/TypeScript
- Java, Kotlin, C#, C/C++, Swift, Objective‑C
- Ruby, PHP, Scala, Elixir, Erlang
- Dart, Lua, R, Julia
- Haskell, OCaml, Clojure

Framework validation repos (examples; use active OSS projects per language):

- JS/TS: React, Next.js, Vue, Svelte
- Python: Django, FastAPI
- Java: Spring
- Ruby: Rails
- PHP: Laravel
- Go: Gin or Echo
- C#: ASP.NET Core
- Swift: Vapor

## Documentation Updates

- `SKILL.md`: specify when to run `cortex index` or daemon `/index/refresh`, and `watch` for aggressive live updates.
- README: add language support matrix, benchmark table, and compaction guarantees.
