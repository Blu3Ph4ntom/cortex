---
name: cortex
description: Working guide for contributors and coding agents in the Cortex repository. Use when changing the Rust graph engine, CLI, daemon, installers, docs, or Zola site.
---

# Cortex Repository Guide

## What This Repo Is

Cortex is a local-first code knowledge engine for AI agents and developers. It indexes a repository into a persistent semantic graph and exposes that graph through:

- `cortex-core`: graph model, extractors, indexer, storage, query engine
- `cortex-cli`: local CLI
- `cortex-daemon`: local HTTP daemon
- `site/`: Zola single-page website

## Working Rules

- Preserve the public positioning: useful OSS beta, not compiler-grade semantic truth.
- Keep the repo local-first. Do not add hosted-service assumptions to the core product.
- Treat the website as a single-page docs and landing surface. Avoid reintroducing separate docs UX.
- Keep install commands real. Any public docs change should reflect commands that were actually validated.

## Core Commands

Use the MSVC toolchain on this Windows machine:

```powershell
cargo +stable-x86_64-pc-windows-msvc test --all-targets
cargo +stable-x86_64-pc-windows-msvc clippy --all-targets --all-features -- -D warnings
```

For self-host verification:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\self-test.ps1
```

## Website Commands

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

## Content Standards

- README should read like public developer documentation.
- Site copy should be concrete, not hype-heavy.
- Any claims about usefulness should come from actual runs against this repository or other tested repos.
- Keep the left rail and section rhythm aligned with the `port0` website model.

## Release Surface

If changing install or release behavior, also inspect:

- `.github/workflows/release.yml`
- `scripts/install.sh`
- `scripts/install.ps1`
- `README.md`

Do not ship install instructions that were not tested.
