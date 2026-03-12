---
name: cortex
description: Production-oriented repository skill for Cortex. Use when modifying the graph engine, CLI, daemon, installers, or single-page Zola site.
---

# Cortex Skill

Use this skill when working inside the Cortex repository.

## What Cortex Is

Cortex is a local-first code knowledge engine for AI agents and developers. It indexes a repository into a persistent semantic graph and exposes that graph through:

- `cortex-core`
- `cortex-cli`
- `cortex-daemon`
- `site/`

## Working Rules

- Preserve the public positioning: useful OSS beta, not compiler-grade semantic truth.
- Keep the repo local-first.
- Keep the website single-page with section navigation.
- Only terminal and API command blocks should get copy buttons in the UI.
- Keep README and site copy public-facing and concrete.

## Repository Map

- `crates/cortex-core`: graph model, extractors, indexer, store, query engine
- `crates/cortex-cli`: CLI interface
- `crates/cortex-daemon`: local HTTP daemon
- `site/`: Zola marketing/docs surface
- `scripts/`: install and self-test scripts

## Validation Commands

Use the MSVC toolchain on this Windows machine:

```powershell
cargo +stable-x86_64-pc-windows-msvc test --all-targets
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets --all-features -- -D warnings
```

For self-host verification:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\self-test.ps1
```

## Website Workflow

Zola is installed via WinGet and is not guaranteed to be on `PATH`. Use:

```powershell
& 'C:\Users\heman\AppData\Local\Microsoft\WinGet\Packages\getzola.zola_Microsoft.Winget.Source_8wekyb3d8bbwe\zola.exe' build
```

From `site/`, local preview:

```powershell
& 'C:\Users\heman\AppData\Local\Microsoft\WinGet\Packages\getzola.zola_Microsoft.Winget.Source_8wekyb3d8bbwe\zola.exe' serve --interface 127.0.0.1 --port 1111
```

## Browser Verification

Use `agent-browser` for visual checks when changing the site.

Recommended local flow:

```powershell
agent-browser --session cortex open http://127.0.0.1:1111
agent-browser --session cortex screenshot --full
```

If Playwright’s default browser binary is missing, pass the installed Chromium explicitly:

```powershell
agent-browser --session cortex --executable-path "$env:LOCALAPPDATA\ms-playwright\chromium-1208\chrome-win64\chrome.exe" open http://127.0.0.1:1111
```

## Release Surface

If changing install or release behavior, inspect:

- `.github/workflows/release.yml`
- `scripts/install.sh`
- `scripts/install.ps1`
- `README.md`

Do not ship install instructions that were not tested.
