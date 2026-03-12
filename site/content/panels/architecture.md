+++
title = "A small Rust monorepo with hard boundaries."
description = "The codebase is split so the core graph engine stays reusable while the CLI and daemon stay thin."
[extra]
section_id = "architecture"
nav_title = "Architecture"
eyebrow = "Architecture"
edit_path = "site/content/panels/architecture.md"
+++

<div class="features">
  <div class="feature-item">
    <h4><code>cortex-core</code></h4>
    <p>Graph model, parser extractors, indexer, storage layer, and typed query engine.</p>
  </div>
  <div class="feature-item">
    <h4><code>cortex-cli</code></h4>
    <p>Index, doctor, export, watch, and query commands for local use and scripting.</p>
  </div>
  <div class="feature-item">
    <h4><code>cortex-daemon</code></h4>
    <p>Local HTTP surface for tools that want a long-lived process instead of spawning the CLI for each lookup.</p>
  </div>
  <div class="feature-item">
    <h4>Current semantic shape</h4>
    <p>Repositories, files, symbols, and edges such as defines, contains, imports, references, calls, depends_on, and owned_by.</p>
  </div>
</div>

<div class="fact-list">
  <div class="fact-item">
    <span>Current tradeoff</span>
    <p>Cortex is intentionally conservative and syntax-driven. That makes it useful for navigation and pre-edit context, but not compiler-grade proof.</p>
  </div>
</div>
