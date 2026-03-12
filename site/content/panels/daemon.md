+++
title = "Keep the graph warm for tools."
description = "Use the local daemon when another process needs repeated structural access without shelling out for each query."
[extra]
section_id = "daemon"
nav_title = "Daemon API"
eyebrow = "Daemon API"
edit_path = "site/content/panels/daemon.md"
+++

<div class="quick">
  <div class="command-card">
    <span>Start the daemon</span>
    <code>cortexd --repo /path/to/repo --bind 127.0.0.1:8787</code>
  </div>
  <div class="command-card">
    <span>Resolve a symbol over HTTP</span>
    <code>curl "http://127.0.0.1:8787/graph/find_symbol?name=RepositorySession"</code>
  </div>
  <div class="command-card">
    <span>Ask for callers over HTTP</span>
    <code>curl "http://127.0.0.1:8787/graph/callers?target=open_session"</code>
  </div>
</div>
