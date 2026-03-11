+++
title = "Getting Started"
weight = 1
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

On Windows, use the MSVC toolchain if the GNU toolchain does not have `gcc.exe` and `dlltool.exe`.

## First Index

```bash
cortex index --repo /path/to/repo
```

## First Query

```bash
cortex query --repo /path/to/repo find-symbol --name main
```

## Isolated Store

The embedded graph store is single-writer. If you want repeatable runs or concurrent local usage, point each run at a dedicated store path:

```bash
cortex index --repo /path/to/repo --store-path /tmp/cortex-store
```
