# Project Goal: Workflow Step Editor

## Overview

Cloneable.ai builds desktop agents that learn to automate enterprise software
by watching humans work. This project covers one part of that problem space.
Build a tool that defines and visualizes automation workflow steps.

The project has an intentionally ambitious scope. Completion of every item is
not expected. Prioritization, tradeoffs, and decision communication are part of
the evaluation.

## Time limit

Spend no more than four hours on this project.

## Evaluation criteria

1. **AI tool fluency**
   - Use AI coding assistants extensively.
   - Show effective prompting and iteration.
   - Identify when AI output needs human refinement.
   - Use AI tools to research the problem and apply the findings.
2. **Prioritization under constraints**
   - Choose what to build first.
   - State what was removed from scope and why.
3. **Code quality at speed**
   - Produce code that is clean enough to extend and pragmatic enough to ship.
   - Avoid unnecessary engineering.
4. **Technical communication**
   - Provide a clear written summary of the work.

## Product goal

Build a Tauri desktop application with a Rust backend. The application lets
users define automation workflow steps by capturing desktop click events. It
also lets users review each captured step.

## Core requirements

### Must have

- Build a Tauri desktop application.
- Provide a frontend button that starts workflow recording.
- Use a Rust backend to monitor clicks and keyboard entries.
- Capture three screenshots for each event:
  - Full screen.
  - Window crop.
  - Click crop of the relevant UI element.

### Should have

- Parse captured actions into understandable steps.
- Let users add text titles and descriptions to steps.
- Classify each annotation as `click`, `type`, `wait`, or `assert`.

### Nice to have

- Group related text input into one action.
- Support keyboard shortcuts.

## Technical constraints

- **Frontend:** Use Tauri with any JavaScript or TypeScript framework.
  React, Svelte, Vue, and vanilla JavaScript are acceptable.
- **Backend:** Use Rust and Tauri commands. Do not add an external server.
- **Storage:** Use the local filesystem. JSON is acceptable. SQLite is also
  acceptable.
- **Services:** Keep all application behavior local. Require no API keys.

## Deliverables

1. Provide a working application in a GitHub repository.
   - The application must run with standard Tauri build commands.
2. Provide a `README.md` with:
   - Setup instructions.
   - Completed and removed-scope items.
   - Key technical decisions and tradeoffs.
   - Work that would follow with more time.
   - AI tools used during the project.
   - What worked and what did not work with those tools.
   - How AI supported planning, research, implementation, and review.
3. Provide a walkthrough that is no longer than five minutes.
   - Use Loom, an unlisted YouTube video, or a written walkthrough.
   - Show the application and explain the main decisions.

## AI usage summary requirements

The final `README.md` must include:

- The AI tools used.
- One specific example where AI accelerated the work.
- One specific example where AI produced a poor result or required a major
  correction.

Example prompts or chat excerpts can appear in a `/prompts` directory or an
appendix. This material is optional.

## Human submission reference

The human submission consists of:

- A link to the GitHub repository.
- A link to the walkthrough.
- Any questions or clarifications found during the project.

The original brief states a due date of five business days after receipt. It
also lists `tyler@cloneable.ai` for questions and states that clarification
questions are encouraged.
