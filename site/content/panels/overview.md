+++
title = "Know the code before you edit."
description = "Cortex indexes a repository into a local semantic graph so agents can resolve ownership, inspect callers, trace dependencies, and estimate blast radius before they edit code."
[extra]
section_id = "overview"
nav_title = "Overview"
eyebrow = "Local code knowledge for agents"
edit_path = "site/content/panels/overview.md"
+++

<div class="features">
  <div class="feature-item">
    <h4>Answer structural questions directly</h4>
    <p>Resolve who owns a symbol, where it is used, what calls it, and what sits in its immediate dependency neighborhood without walking raw grep output.</p>
  </div>
  <div class="feature-item">
    <h4>Keep everything local</h4>
    <p>Cortex builds and queries a persistent graph on your machine. No hosted indexing service, background sync, or remote vendor dependency is required.</p>
  </div>
  <div class="feature-item">
    <h4>Designed for agent workflows</h4>
    <p>Use the CLI for one-off orientation or keep the daemon warm for editors, automations, and coding agents that need repeated graph lookups.</p>
  </div>
  <div class="feature-item">
    <h4>Grounded in actual measurements</h4>
    <p>The repository ships a reproducible benchmark harness and published artifacts comparing Cortex against a raw text-search baseline on real open-source repositories.</p>
  </div>
</div>
