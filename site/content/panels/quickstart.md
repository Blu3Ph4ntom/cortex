+++
title = "Index once, query before you edit."
description = "The practical loop is simple: build the graph, resolve the owner, inspect callers, then check likely blast radius."
[extra]
section_id = "quickstart"
nav_title = "Quick start"
eyebrow = "Quick start"
edit_path = "site/content/panels/quickstart.md"
+++

<div class="quick">
  <div class="command-card">
    <span>Index a repository</span>
    <code>cortex index --repo /path/to/repo</code>
  </div>
  <div class="command-card">
    <span>Resolve a symbol owner</span>
    <code>cortex query --repo /path/to/repo find-symbol --name RepositorySession</code>
  </div>
  <div class="command-card">
    <span>Check callers before a change</span>
    <code>cortex query --repo /path/to/repo callers --target open_session</code>
  </div>
  <div class="command-card">
    <span>Inspect dependencies and impact</span>
    <code>cortex query --repo /path/to/repo dependencies --target RepositorySession --direction both --depth 1
cortex query --repo /path/to/repo impact --target open_session --depth 1</code>
  </div>
</div>
