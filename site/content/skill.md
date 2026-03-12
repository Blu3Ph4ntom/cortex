+++
title = "Cortex Skill"
template = "skill-page.html"
description = "Install and use the Cortex agent skill for structural codebase reasoning."
+++

## Install the skill

Install the Cortex skill into a compatible agent runtime from the public GitHub repository:

```bash
npx skills add https://github.com/Blu3Ph4ntom/cortex --skill cortex
```

After installation, make sure `cortex` and `cortexd` are available on the agent host. The skill assumes the Cortex CLI is installed and callable in the working environment.

## Direct skill URLs

If your agent runner accepts remote skill files directly, use one of these:

- GitHub page: [github.com/Blu3Ph4ntom/cortex/blob/main/SKILL.md](https://github.com/Blu3Ph4ntom/cortex/blob/main/SKILL.md)
- Raw file: [raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/SKILL.md](https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/SKILL.md)

## Tell an agent how to use it

The practical loop is:

```bash
cortex index --repo /path/to/repo
cortex query --repo /path/to/repo find-symbol --name PaymentService
cortex query --repo /path/to/repo callers --target update_status
cortex query --repo /path/to/repo impact --target update_status --depth 1
```

Give the agent this expectation:

- use Cortex before broad grep when structural context matters
- quote the resolved file path and line span in reasoning
- distinguish confirmed graph structure from inferred runtime behavior
- fall back to direct code reading when dynamic dispatch or framework behavior is unclear

## Cortex Skill

Use this skill when Cortex is available in the working environment and you need structural answers about the codebase before making changes.

### Best For

- finding the real owner of a symbol
- checking who calls a function before editing it
- tracing inbound and outbound dependencies
- estimating bounded impact before refactors
- giving coding agents more reliable architectural context than text search alone

### Fast Workflow

1. Build or refresh the local graph:

```bash
cortex index --repo /path/to/repo
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

### What Each Query Is Good At

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

### Daemon Mode

If repeated queries are needed, run the daemon once and hit the HTTP API:

```bash
cortexd --repo /path/to/repo --bind 127.0.0.1:8787
curl "http://127.0.0.1:8787/graph/find_symbol?name=PaymentService"
```

### How To Use Results Well

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
