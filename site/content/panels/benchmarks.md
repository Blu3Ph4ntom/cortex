+++
title = "Measured against raw text search."
description = "The benchmark harness in this repository compares Cortex warm-query results against `git grep -n -w` on the same structural tasks."
[extra]
section_id = "benchmarks"
nav_title = "Benchmarks"
eyebrow = "Benchmarks"
edit_path = "site/content/panels/benchmarks.md"
+++

<div class="fact-list">
  <div class="fact-item">
    <span>What is being measured</span>
    <p>Cold indexing on a fresh local store, warm structural queries on a prepared store, and a raw grep baseline over the same repositories and targets.</p>
  </div>
  <div class="fact-item">
    <span>How to read the results</span>
    <p>Grep is often faster to start, but Cortex wins by collapsing the search surface into a much smaller set of structural answers that an agent can actually use.</p>
  </div>
</div>
