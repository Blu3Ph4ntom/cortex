+++
title = "Use it where agents usually guess."
description = "Cortex is strongest before edits, during refactors, and when review context needs real structure instead of filename heuristics."
[extra]
section_id = "agent-workflows"
nav_title = "Agent workflows"
eyebrow = "Agent workflows"
edit_path = "site/content/panels/agent-workflows.md"
+++

<div class="features">
  <div class="feature-item">
    <h4>Pre-edit orientation</h4>
    <p>Resolve the canonical owner of a symbol before an agent starts patching an unfamiliar codebase.</p>
  </div>
  <div class="feature-item">
    <h4>Refactor safety checks</h4>
    <p>Inspect callers and dependencies before changing a signature or behavior that might ripple across the repository.</p>
  </div>
  <div class="feature-item">
    <h4>Review context</h4>
    <p>Attach structural neighbors and likely blast radius to a patch review so the summary is grounded in code relationships.</p>
  </div>
  <div class="feature-item">
    <h4>Local automation</h4>
    <p>Keep a daemon warm for repeated lookups from editor tooling, local automations, and agent loops.</p>
  </div>
</div>

<div class="fact-list">
  <div class="fact-item">
    <span>Good fit</span>
    <p>Repository orientation, targeted edits, change review, bounded impact checks, and machine-readable architecture discovery.</p>
  </div>
  <div class="fact-item">
    <span>Not a fit</span>
    <p>Compiler-grade proof, runtime data-flow certainty, or heavy framework indirection that only resolves through execution.</p>
  </div>
</div>
