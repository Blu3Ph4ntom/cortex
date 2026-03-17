# Benchmark Expansion Design

Date: 2026-03-17

## Context & Goals
- Expand benchmark coverage to 20+ languages/frameworks using real open-source repositories.
- Auto clone/pull benchmark repositories into a gitignored local path.
- Run cold index and warm query medians and regenerate:
  - `benchmarks/latest.md`
  - `benchmarks/latest.json`
  - `site/data/benchmarks.json`
- Update README benchmark snapshot after runs.
- Maintain the guarantee that Cortex stores only the current live index.
- Quality bar: first-party extractor languages must be “great.” Non-first-party languages should still be “good” and better than peers.

## Requirements
- Use existing PowerShell harness `scripts/benchmark.ps1`.
- Prefer in-script matrix (no new manifest files) to avoid extra artifacts.
- Benchmark repo clones must be local and gitignored.
- Keep baseline comparison `git grep -n -w`.
- Fail fast if repo clone/update or indexing fails.

## Approach Options
1) **Inline repo/scenario matrix in benchmark script (recommended)**
   - Expand `$repos`/`$scenarios` blocks in `scripts/benchmark.ps1`.
   - Add `repoRoot` and `Ensure-Repo` loop to clone/pull before benchmarking.
   - Pros: minimal changes, no new files. Cons: larger script.

2) **External manifest file (JSON/YAML)**
   - Move repo/scenario definitions to a data file.
   - Pros: easier edits. Cons: adds a new file (not preferred).

3) **Per-language modules**
   - Split benchmark definition per language in separate scripts.
   - Pros: modular. Cons: unnecessary complexity.

**Recommendation**: Option 1 for minimal change and simplicity.

## Proposed Design
### Data Model (in-script)
- Add `$repoRoot` (gitignored path, e.g., `benchmarks/repos` or existing fieldtest root).
- Replace `$repos` with `$repoSpecs`:
  - `key`, `name`, `language`, `url`, `path`.
- Expand `$scenarios` with at least one structural query per repo.
  - Use `find_symbol` for ownership.
  - Use `callers` for call graph proof.

### Workflow
1) **Preflight**: verify release binary exists.
2) **Ensure directories**: `benchRoot`, `siteDataRoot`, `runStoresRoot`, `repoRoot`.
3) **Ensure repos**: `Ensure-Repo` clone/pull into `repoRoot`.
4) **Index**: cold index runs (fresh store per iteration), then warm store.
5) **Queries**: run scenario queries + grep baseline for N iterations.
6) **Cleanup**: remove active stores.
7) **Write artifacts**: `latest.json`, `site/data/benchmarks.json`, `latest.md`.
8) **Update README** with the new benchmark snapshot.

### Error Handling
- Any non-zero command exit fails the run.
- Repo missing or clone failure aborts.
- Index failures abort.

### Quality Bar (extractors)
- For first-party languages, aim for full symbol/relationship extraction with high-quality results.
- For non-first-party languages, ensure extraction still returns meaningful structure and outperforms raw text search.

## Test Plan
- Run `powershell -ExecutionPolicy Bypass -File .\scripts\benchmark.ps1`.
- Verify artifacts updated and `latest.md` matches expectations.
- Update README benchmark snapshot accordingly.

## Open Questions (None)
All requirements captured.
