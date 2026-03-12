+++
title = "Install the skill, then let agents use structure."
description = "The website renders the human-facing guide here. The repository root keeps the agent-facing `SKILL.md` that coding agents can install and use directly."
[extra]
section_id = "skill"
nav_title = "Skill"
eyebrow = "Agent skill"
edit_path = "site/content/panels/skill.md"
+++

<div class="grid two-up">
  <div>
    <h3>Install the skill</h3>
    <div class="quick">
      <div class="command-card">
        <span>Skill installer</span>
        <code>npx skills add https://github.com/Blu3Ph4ntom/cortex --skill cortex</code>
      </div>
      <div class="command-card">
        <span>Direct raw file</span>
        <code>https://raw.githubusercontent.com/Blu3Ph4ntom/cortex/main/SKILL.md</code>
      </div>
    </div>
  </div>

  <div>
    <h3>Tell an agent to use it</h3>
    <div class="quick">
      <div class="command-card">
        <span>Instruction snippet</span>
        <code>Use the Cortex skill before editing this repository. Index the repo, resolve the owner of the target symbol, inspect callers or dependencies, then run impact before you patch code.</code>
      </div>
    </div>
  </div>
</div>

<div class="fact-list">
  <div class="fact-item">
    <span>What the root <code>SKILL.md</code> teaches</span>
    <p>When to use Cortex, which queries to sequence first, how to quote structural results, and how to report uncertainty honestly when runtime behavior exceeds syntax.</p>
  </div>
  <div class="fact-item">
    <span>Why this section exists</span>
    <p>Humans can review the workflow here on the website, while coding agents can consume the real source file directly from the repository.</p>
  </div>
</div>

<div class="feature-item skill-source-card">
  <h4>Agent-facing source file</h4>
  <p><a class="external" href="https://github.com/Blu3Ph4ntom/cortex/blob/main/SKILL.md" rel="noopener" target="_blank">github.com/Blu3Ph4ntom/cortex/blob/main/SKILL.md</a></p>
</div>
