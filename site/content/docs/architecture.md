+++
title = "Architecture"
weight = 3
description = "Understand the Rust monorepo, the indexing pipeline, and the graph shape Cortex serves today."
+++

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

## Design Intent

The project is deliberately local-first:

- no external graph database
- no hosted dependency
- no cloud control plane
- no multi-repo global model in v1

The goal is to make structural repository memory cheap to run beside an AI agent or editor.

## Current Tradeoff

Today’s implementation prioritizes deterministic structural usefulness over semantic completeness. That means the graph is good for navigation, review context, and bounded impact analysis, while deeper compiler-grade guarantees remain future work.
