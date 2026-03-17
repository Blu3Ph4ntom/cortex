---
name: cortex
description: Use Cortex to get structural codebase context before editing an unfamiliar repository. Index the repo locally, resolve symbol ownership, trace callers and dependencies, and estimate blast radius through the CLI or local daemon.
---

# Cortex Skill

Use this skill when Cortex is available in the working environment and you need structural answers about the codebase before making changes.

## Best For

- finding the real owner of a symbol
- checking who calls a function before editing it
- tracing inbound and outbound dependencies
- estimating bounded impact before refactors
- giving coding agents more reliable architectural context than text search alone

## Fast Workflow

1. Build or refresh the local graph:

```bash
cortex index --repo /path/to/repo
```

If the repo changes after indexing, refresh the graph before answering:

```bash
cortex index --repo /path/to/repo
```

If using the daemon, refresh over HTTP:

```bash
curl -X POST "http://127.0.0.1:8787/index/refresh"
```

2. Resolve the target symbol:

```bash
cortex query --repo /path/to/repo find-symbol --name PaymentService
```

3. Inspect structural neighborhood before editing:

```bash
cortex query --repo /path/to/repo callers --target update_status
cortex query --repo /path/to/repo dependencies --target PaymentService --direction both --depth 1
cortex query --repo /path/to/repo impact --target update_status --depth 1
```

## What Each Query Is Good At

- `find-symbol`
  Use when you need the canonical definition candidate and file/span location.
- `callers`
  Use before changing behavior or signatures to see likely upstream dependents.
- `callees`
  Use when reading through behavior from a function outward.
- `dependencies`
  Use when you need the local structural neighborhood around a symbol or file.
- `references`
  Use for best-effort identifier references when call edges are insufficient.
- `impact`
  Use for a conservative blast-radius estimate with supporting edges.
- `explain`
  Use when you want a compact summary of why a symbol matters structurally.

## Daemon Mode

If repeated queries are needed, run the daemon once and hit the HTTP API:

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
curl "http://127.0.0.1:8787/graph/find_symbol?name=PaymentService"
```

## How To Use Results Well

- quote the resolved file path and line span in your reasoning
- distinguish confirmed structure from inferred behavior
- use `callers`, `dependencies`, and `impact` before edits that could widen blast radius
- fall back to direct code reading when the graph is too broad or ambiguous

## Current Limits

- Cortex is conservative and syntax-driven, not compiler-grade truth
- dynamic dispatch, runtime data flow, and framework magic may need manual confirmation
- larger impact results may include tests and example code; filter accordingly

## Good Output Pattern For Agents

When reporting findings from Cortex, include:

- the queried symbol or target
- the resolved owner path and span
- the most relevant callers or dependencies
- what remains uncertain and needs manual reading
