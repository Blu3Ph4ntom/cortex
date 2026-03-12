+++
title = "Install Cortex."
description = "Use the one-line installer for a release binary, or build from source if you want to work on the monorepo."
[extra]
section_id = "install"
nav_title = "Install"
eyebrow = "Install"
edit_path = "site/content/panels/install.md"
+++

<div class="quick">
  <div class="command-card">
    <span>macOS / Linux</span>
    <code>curl -fsSL https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.sh | sh</code>
  </div>
  <div class="command-card">
    <span>Windows (PowerShell)</span>
    <code>irm https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/scripts/install.ps1 | iex</code>
  </div>
  <div class="command-card">
    <span>Build from source</span>
    <code>git clone https://github.com/Blu3Ph4ntom/cortex.git
cd cortex
cargo install --path crates/cortex-cli
cargo install --path crates/cortex-daemon</code>
  </div>
</div>

<div class="fact-list">
  <div class="fact-item">
    <span>Binary locations</span>
    <p>The release installers place binaries in a common local path: <code>~/.local/bin</code> on Unix and <code>$HOME\.cortex\bin</code> on Windows.</p>
  </div>
  <div class="fact-item">
    <span>Windows PATH behavior</span>
    <p>The PowerShell installer adds the default install directory to the current session PATH immediately and persists it for new shells. If you set <code>CORTEX_INSTALL_DIR</code>, it uses that directory but leaves your user PATH unchanged.</p>
  </div>
  <div class="fact-item">
    <span>Windows toolchain</span>
    <p>Build from source with the MSVC Rust toolchain if your GNU installation does not include <code>gcc.exe</code> and <code>dlltool.exe</code>.</p>
  </div>
</div>
