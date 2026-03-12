+++
title = "A small query surface on purpose."
description = "Cortex keeps the query model narrow so agents can learn it quickly and reuse it across repositories."
[extra]
section_id = "queries"
nav_title = "Query model"
eyebrow = "Query model"
edit_path = "site/content/panels/queries.md"
+++

<div class="features">
  <div class="feature-item">
    <h4><code>find-symbol</code></h4>
    <p>Resolve the canonical owner candidate for a symbol and get back a file path plus span.</p>
  </div>
  <div class="feature-item">
    <h4><code>callers</code> / <code>callees</code></h4>
    <p>Walk the call chain before you change behavior or when you are tracing outward from a function.</p>
  </div>
  <div class="feature-item">
    <h4><code>dependencies</code></h4>
    <p>Inspect the local inbound and outbound structural neighborhood around a symbol or file.</p>
  </div>
  <div class="feature-item">
    <h4><code>references</code> / <code>impact</code> / <code>explain</code></h4>
    <p>Get conservative reference sets, likely blast radius, and a summary of why a symbol matters in the current graph.</p>
  </div>
</div>

<div class="quick">
  <div class="command-card">
    <span>Owner lookup</span>
    <code>cortex query --repo /path/to/repo find-symbol --name PaymentService</code>
  </div>
  <div class="command-card">
    <span>Caller trace</span>
    <code>cortex query --repo /path/to/repo callers --target update_status</code>
  </div>
  <div class="command-card">
    <span>Explain a symbol</span>
    <code>cortex query --repo /path/to/repo explain --target PaymentService</code>
  </div>
</div>
