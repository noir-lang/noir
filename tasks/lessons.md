# Lessons

- Before adding GitHub-to-ClaudeBox infrastructure, search the configured
  repositories for an existing dispatcher, kickoff helper, and task-specific
  skill. The warning sign is a workflow beginning to duplicate `/run`
  authentication, routing, or standing instructions that another repository
  already centralizes.
- Reusing a shared dispatcher does not require delegating the behavior contract
  to a repository skill. When the operator wants the automation to be
  self-contained, pass the full standing prompt through the dispatcher and use
  skills only as optional implementation aids.
- A repository's CI should not dispatch an unrelated repository's workflow to
  reach a service. Pull the generic client into the owning repository as a
  local action, so the dependency and required secrets are explicit and local.
- Keep domain-specific recovery policy out of generic actions. Let the action
  fail clearly, then have the calling workflow select its own fallback based on
  the action step's outcome.
- In a composite action, separate input validation, external setup, and the
  final service call into named steps. This makes the Actions UI identify the
  failing phase without fragmenting every individual assertion into a step.
