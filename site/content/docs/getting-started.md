+++
title = "Getting Started"
weight = 1
description = "Install Cortex, build your first local index, and run the first useful queries."
+++

# Getting Started

Cortex ships as a Rust monorepo with two user-facing binaries:

- `cortex`: local CLI
- `cortexd`: local HTTP daemon

## Install

```bash
cargo install --path crates/cortex-cli
cargo install --path crates/cortex-daemon
```

On Windows, prefer the MSVC toolchain if the GNU toolchain does not provide `gcc.exe` and `dlltool.exe`.

## What You Need

- Rust stable
- a local clone of the repository you want to index
- a writable store path if you want isolated or repeatable runs

The graph store is embedded and local. You do not need a hosted service or external database to get started.

## First Index

```bash
cortex index --repo /path/to/repo
```

This walks the repository, extracts normalized symbols and relations, and persists the graph locally.

## First Query

```bash
cortex query --repo /path/to/repo find-symbol --name main
```

Good first queries:

- `find-symbol` when you know a symbol name
- `callers` before changing a helper
- `dependencies` when you need immediate structural neighborhood
- `explain` when you want a compressed summary for an agent prompt

## Isolated Store

The embedded graph store is single-writer. If you want repeatable runs, CI isolation, or concurrent local usage, point each run at a dedicated store path:

```bash
cortex index --repo /path/to/repo --store-path /tmp/cortex-store
```

## Run the Daemon

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
```

This exposes the same local graph through an HTTP interface so an editor, agent runtime, or external tool can query Cortex without shelling out for every request.

## Readiness

Cortex is already useful for local structural awareness and agent context building. It is not yet compiler-grade whole-program analysis, and the current graph should be treated as conservative rather than semantically perfect.
