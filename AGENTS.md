# AGENTS.md

## Purpose
This file tells coding agents how to work in this project.

The goal is to make sound long-term decisions, explain concepts clearly, and help the user learn while building reliable software.

## Project Context
Scribblenot is a local-first Rust terminal app for building structured clinical notes from configurable YAML data.

It uses `ratatui` and `crossterm` for the TUI. Treat `data/*.yml` as product data, not test fixtures. Preserve the configurable section-driven design, and do not assume this is a generic CRUD or web app.

Use existing `operations/` notes and validation docs when relevant.

## Collaboration Style
The user is a self-taught enthusiast and benefits from clear, manageable explanations.

Agents should:
- explain in plain English first
- follow with technical detail when useful
- prefer step-by-step guidance
- avoid long dense lists when a simpler explanation will do

When a decision is needed, present:
- two options
- pros and cons for each
- a recommendation
- a short explanation

## Autonomy
When the user is present:
- stop and ask before proceeding when there is uncertainty
- do not make important judgment calls silently

When the user explicitly says they will be away:
- use reasonable judgment to continue
- avoid unnecessary risk
- document important decisions made independently

## Scope Boundaries
Do not read from, write to, or modify anything outside:
- `C:\Users\solar\.codex`
- `C:\Users\solar\Documents\Claude Projects`

Anything outside those folders requires explicit permission.

## Code Quality
Prefer long-term success over quick patches.

If surrounding code is weak or a broader rewrite looks justified, do not silently perform broad cleanup. Propose the plan first, explain what seems wrong in plain language, discuss possible intent, and get agreement before making larger structural changes.

## Behavior Changes
If a change affects what the software does from the user's point of view:
- update relevant tests automatically when they exist
- update relevant documentation automatically
- clearly explain what changed

## Validation
Validation should be practical and improve over time.

Agents should:
- recommend sensible validation steps for this project
- favor practices that improve reliability and security
- clearly say what was and was not verified

If automated tests are missing:
- use any documented manual checks or validation notes that already exist
- clearly say that automated coverage is missing
- ask whether to proceed now or discuss stronger checks first when the risk is meaningful
- recommend reasonable next steps

## Risk and Safety
Stop and ask before making changes involving:
- architecture
- public behavior
- authentication or permissions
- secrets, API keys, tokens, or environment files
- billing, payments, or subscriptions
- database schema changes or migrations
- deployment, infrastructure, CI/CD, or hosting configuration
- file deletion, bulk moves, or destructive scripts
- security-sensitive logic
- anything difficult to undo
- any other high-risk or high-impact area

Normal bug fixes, focused refactors, documentation updates, and low-risk validation do not require extra approval unless they touch one of the sensitive areas above.

If a security concern is noticed:
- stop immediately
- explain the risk clearly
- do not silently patch it without discussion unless explicitly directed

## Progress Updates
Provide updates at meaningful milestones rather than constant low-level commentary.

## Continuous Improvement
Keep a running list of improvement ideas in `roadmap.md`.

Surface improvements when relevant and actively propose useful next steps over time.
