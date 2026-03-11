+++
title = "Queries"
weight = 2
+++

# Queries

Cortex currently exposes these machine-readable query families:

- `find-symbol`
- `dependencies`
- `callers`
- `callees`
- `references`
- `impact`
- `explain`

## Example: Find a Type

```bash
cortex query --repo . find-symbol --name RepositorySession
```

## Example: Callers Before a Change

```bash
cortex query --repo . callers --target open_session
```

## Example: Structural Neighborhood

```bash
cortex query --repo . dependencies --target RepositorySession --direction both --depth 1
```

## Example: Conservative Impact

```bash
cortex query --repo . impact --target open_session --depth 1
```

The current graph is conservative and syntax-driven. It is best for structural awareness, not compiler-grade proof.
