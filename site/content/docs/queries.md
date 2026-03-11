+++
title = "Queries"
weight = 2
description = "Use typed queries to ask Cortex about symbols, callers, dependencies, references, and impact."
+++

Cortex currently exposes these machine-readable query families:

- `find-symbol`
- `dependencies`
- `callers`
- `callees`
- `references`
- `impact`
- `explain`

Each query is designed to answer a structural question that plain text search handles poorly.

## Find a Type

```bash
cortex query --repo . find-symbol --name RepositorySession
```

Use this when you know the symbol name but not its real owner. On this repository, `RepositorySession` resolves to the core indexer layer instead of returning a wall of incidental references.

## Callers Before a Change

```bash
cortex query --repo . callers --target open_session
```

Use this before changing a helper or internal API. On this repository, `open_session` surfaces the CLI entrypoints that actually invoke it.

## Structural Neighborhood

```bash
cortex query --repo . dependencies --target RepositorySession --direction both --depth 1
```

Use this when you need immediate inbound and outbound neighbors around a symbol.

## Conservative Impact

```bash
cortex query --repo . impact --target open_session --depth 1
```

Use this as a bounded blast-radius estimate before refactors or automated edits.

## Explain

```bash
cortex query --repo . explain --target RepositorySession
```

`explain` is useful when you want a compressed human-readable summary that can be passed into an agent context window.

## Limits

The current graph is conservative and syntax-driven. It is strong for structural awareness, navigation, and pre-edit context. It should not yet be treated as compiler-grade proof.
