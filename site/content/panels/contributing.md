+++
title = "Treat it like infrastructure."
description = "Contributor ergonomics, repeatable tests, and benchmark artifacts matter because Cortex is meant to support real engineering workflows."
[extra]
section_id = "contributing"
nav_title = "Contributing"
eyebrow = "Contributing"
edit_path = "site/content/panels/contributing.md"
+++

<div class="grid two-up">
  <div>
    <h3>Developer loop</h3>
    <div class="quick">
      <div class="command-card">
        <span>Format, lint, test, and self-check</span>
        <code>cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
powershell -ExecutionPolicy Bypass -File .\scripts\self-test.ps1</code>
      </div>
    </div>
  </div>

  <div>
    <h3>What ships with the repo</h3>
    <div class="fact-list compact">
      <div class="fact-item">
        <span>AGENTS.md</span>
        <p>Repository-specific contributor instructions for working on Cortex itself.</p>
      </div>
      <div class="fact-item">
        <span>SKILL.md</span>
        <p>Agent-facing instructions for using Cortex on arbitrary codebases.</p>
      </div>
      <div class="fact-item">
        <span>Benchmarks</span>
        <p>Reproducible benchmark artifacts and the PowerShell harness that generated them.</p>
      </div>
    </div>
  </div>
</div>

<div class="fact-list">
  <div class="fact-item">
    <span>Project status</span>
    <p>Cortex is already useful for local structural awareness and agent workflows. It is still OSS beta software, not yet a compiler-accurate or fleet-scale autonomous editing platform.</p>
  </div>
</div>
