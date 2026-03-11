+++
title = "Architecture"
weight = 3
+++

# Architecture

Cortex is organized as a Rust monorepo with three crates:

- `cortex-core`
- `cortex-cli`
- `cortex-daemon`

## Core Pipeline

1. Walk repository files
2. Parse supported languages
3. Extract normalized symbols and relations
4. Materialize a semantic graph
5. Persist the graph locally
6. Serve typed queries through CLI or daemon

## Main Subsystems

- Parser adapters
- Semantic normalizer
- Incremental indexer
- Embedded graph store
- Query engine
- Local serving layer

## Current Semantic Shape

Nodes:

- repository
- file
- symbol

Edges:

- contains
- defines
- imports
- references
- calls
- depends_on
- owned_by

This is enough to be useful for navigation, review context, and bounded impact analysis.
