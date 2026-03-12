# Cortex Agent Instructions

## Purpose

Cortex is a local-first code knowledge engine for AI agents and developers. It indexes a repository into a persistent semantic graph and exposes that graph through a CLI and local daemon.

## Repo Areas

- `crates/cortex-core`: graph schema, extractors, indexer, store, query engine
- `crates/cortex-cli`: CLI commands and query UX
- `crates/cortex-daemon`: HTTP API
- `site/`: Zola single-page website and docs surface
- `scripts/`: installers and self-test helpers

## Working Rules

- Keep product claims accurate. Cortex is useful and public, but still a beta.
- Prefer concrete language over hype in README, site copy, and docs.
- Do not reintroduce a multi-page docs UX. The site should remain a single-page surface with section anchors.
- Only make command blocks copyable in the UI. Informational cards should not behave like terminals.
- When changing install or release flows, update docs and verify the commands actually work.

## Validation

Use the MSVC toolchain in this environment:

```powershell
cargo +stable-x86_64-pc-windows-msvc test --all-targets
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets --all-features -- -D warnings
```

For the website:

```powershell
& 'C:\Users\heman\AppData\Local\Microsoft\WinGet\Packages\getzola.zola_Microsoft.Winget.Source_8wekyb3d8bbwe\zola.exe' build
& 'C:\Users\heman\AppData\Local\Microsoft\WinGet\Packages\getzola.zola_Microsoft.Winget.Source_8wekyb3d8bbwe\zola.exe' serve --interface 127.0.0.1 --port 1111
```

Use `agent-browser` for visual verification after meaningful site changes.

## Release Surface

If you touch installation, packaging, or user-facing commands, inspect:

- `README.md`
- `.github/workflows/release.yml`
- `scripts/install.sh`
- `scripts/install.ps1`
- `scripts/self-test.ps1`
