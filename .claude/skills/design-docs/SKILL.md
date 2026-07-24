---
name: design-docs
description: How to write and maintain entries in the `design/` directory. Use when adding, editing, or reviewing a `design/*.md` decision record for the Noir language, compiler, or tooling.
---

# Writing design docs

The `design/` directory records **why** the Noir language, compiler, and tooling work the
way they do. It is a decision record aimed at a contributor, not user-facing documentation
(that lives in `docs/docs`). Follow these rules when writing or editing a `design/*.md`
file.

## Describe the present, not the history

A design doc states the design **as it stands today** and the rationale that holds today.
Do not narrate how the code got here.

- **No historical narrative.** Drop "historically…", "this used to be…", "previously we
  did…", "originally…", "before this change…". Version control already records how things
  changed; the design doc records the decision that is in force now. Rewrite the rationale
  in the present tense as a property the design guarantees, e.g.
  - ❌ "Historically these slugs were bare strings, so a typo silently did nothing. The
    registry fixes that."
  - ✅ "Slugs are a closed enum so the parser can reject an unrecognised slug, rather than
    accepting a typo as an inert no-op."

- **Only keep history that is a live constraint.** The single exception is when a past
  decision still binds the present — a backwards-compatibility guarantee, a migration that
  must remain supported, an on-disk or wire format that cannot change. There the "history"
  is actually a present-day constraint, so state it as one ("The vN artifact format must
  stay readable, so …") rather than as a story about the past.

- **No PR or task progress.** A design doc is not a changelog. Drop "this is the first
  slice", "added in PR #123", "TODO in a follow-up", "the next step will…". Describe the
  intended end-state and the rule for getting there, and link the tracking issue for the
  roadmap. Progress belongs in the PR description and the issue, not in `design/`.

**The cherry-pick test** (mirrors the code-comment rule in `CLAUDE.md`): imagine the file
dropped into a fresh repo with no git history and no memory of the change that introduced
it. Every sentence must still make sense and be verifiable against the current code. If a
sentence only lands for someone who remembers the previous state or the PR that changed it,
cut or rewrite it.

## Keep it accurate and in sync

- Point at the code that implements the decision (module and type names, e.g.
  `compiler/noirc_frontend/src/lint.rs`) so the doc can be checked against reality.
- When a change alters or invalidates a recorded decision, update the corresponding
  `design/` file **in the same PR**. A stale design doc is worse than none.
- Record a new decision here when a change introduces a non-obvious, cross-cutting design
  choice that isn't captured anywhere else.
