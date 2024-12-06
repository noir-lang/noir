---
title: "ACVM crates are not publishable"
assignees: TomAFrench, Savio-Sou
---

The ACVM crates are currently unpublishable, making a release will NOT push our crates to crates.io.

This is likely due to a crate we depend on bumping its MSRV above our own. Our lockfile is not taken into account when publishing to crates.io (as people downloading our crate don't use it) so we need to be able to use the most up-to-date versions of our dependencies (including transient dependencies) specified. 

Check the [MSRV check]({{env.WORKFLOW_URL}}) workflow for details.

This issue was raised by the workflow `{{env.WORKFLOW_NAME}}`
