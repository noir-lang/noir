# Contributing to Noir

Thank you for your interest in contributing to Noir! We value your contributions in making Noir better.

This guide will discuss how the Noir team handles [Commits](#commits), [Pull Requests](#pull-requests), [Merging](#merging), [Releases](#releases), and the [Changelog](#changelog).

__Note:__ We won't force external contributors to follow this verbatim, but following these guidelines definitely helps us in accepting your contributions.

## Commits

We want to keep our commits small and focused. This allows for easily reviewing individual commits and/or splitting up pull requests when they grow too big. Additionally, this allows us to merge smaller changes quicker and release more often.

When committing, it's often useful to use the `git add -p` workflow to decide on what parts of the changeset to stage for commit. When making the commit, write the commit message as a Conventional Commit.

### Conventional Commits

As of the [126ca26](https://github.com/noir-lang/noir/commit/126ca26aaa955bbb002db90308223916a998179f) commit, Noir follows the [Conventional Commits (v1.0.0)](https://www.conventionalcommits.org/en/v1.0.0/) specification. Following this convention allows us to provide an automated release process that also generates a detailed Changelog.

As described by the specification, our commit messages should be written as:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Some examples of this pattern include:

```
feat(syntax): Implement String data type (#123)
```

```
feat: Add support for wasm backend (#234)

Introduces a new backend to nargo that allows the wasm backend to be enabled via a flag.
```

```
feat(syntax): Implement String data type (#123)

Co-authored-by: Blaine <blaine@example.com>
```

The `[optional body]` can also be used to provide more Conventional Commit messages for the Changelog:

```
feat(syntax): Add Boolean type

fix(optimizer): Compile Boolean to u1 type
```

### Conventional Commits: Types

Generally, we want to only use the three primary types defined by the specification:

- `feat:` - This should be the most used type, as most work we are doing in the project are new features. Commits using this type will always show up in the Changelog.
- `fix:` - When fixing a bug, we should use this type. Commits using this type will always show up in the Changelog.
- `chore:` - The least used type, these are **not** included in the Changelog unless they are breaking changes. But remain useful for an understandable commit history.

### Conventional Commits: Breaking Changes

Annotating **BREAKING CHANGES** is extremely important to our release process and versioning. To mark a commit as breaking, we add the `!` character after the type, but before the colon. For example:

```
feat!: Rename nargo build to nargo check (#693)

feat(nargo)!: Enforce minimum rustc version
```

### Conventional Commits: Scopes

Scopes significantly improve the Changelog, so we want to use a scope whenever possible. If we are only changing one part of the project, we can use the name of the crate, like `(nargo)` or `(noirc_driver)`. If a change touches multiple parts of the codebase, there might be a better scope, such as using `(syntax)` for new language features.

```
feat(nargo): Add support for wasm backend (#234)
```

```
feat(syntax): Implement String data type (#123)
```

## Pull Requests

Before you create a pull request, search for any issues related to the change you are making. If none exist already, create an issue that thoroughly describes the problem that you are trying to solve. These are used to inform reviewers of the original intent and should be referenced via the pull request template.

Pull Requests should be focused on the specific change they are working towards. If prerequisite work is required to complete the original pull request, that work should be submitted as a separate pull request.

This strategy avoids scenarios where pull requests grow too large/out-of-scope and don't get proper reviews—we want to avoid "LGTM, I trust you" reviews.

The easiest way to do this is to have multiple Conventional Commits while you work and then you can cherry-pick the smaller changes into separate branches for pull requesting.

### Reviews

For any repository in the noir-lang organization, we require code review & approval by **one** Noir team member before the changes are merged, as enforced by GitHub branch protection. Non-breaking pull requests may be merged at any time. Breaking pull requests should only be merged when the team has general agreement of the changes and is preparing a breaking release.

### With Breaking Changes

Sometimes, we don't merge pull requests with breaking changes immediately upon approval. Since a breaking change will cause Noir to bump to the next "minor" version, we might want to land some fixes in "patch" releases before we begin working on that next breaking version.

## Merging

Once approved by the required number of team members, the pull request can be merged into the `master` branch. Sometimes, especially for external contributions, the final approver may merge the pull request instead of the submitter.

We use "squash merging" to merge all pull requests. This will cause all commits to be combined into one commit—another reason we want to keep pull requests small & focused.

### Squash Merging

When squash merging, we can keep intermediate Conventional Commits by adding them to the body of the commit message; however, the GitHub UI adds a `*` character before each commit message and our releaser bot won't parse that.

When squashing, you want to update both the title of the commit to be a good Conventional Commit and adjust the body to contain any other Conventional Commits that you want to keep (not prefixed with `*`) and delete any extra information. We also keep any "Co-authored-by:" lines at the bottom of the commit if the change was done by multiple authors. If "Co-authored-by:" lines appear due to accepted PR suggestions, it's good to delete them so the author gets full credit for the change.

Our overall approach to squashing is to be mindful of the impact of each commit. The commits populate our Changelog, so it's important to properly convey to Noir consumers what changes have happened. It is also a record that we and others will review in the future. Thus, we want to attribute the change to its correct authors and provide useful information that future contributors need.

For example, given the default squash commit message:

```
feat(syntax): Implement first-class functions (#123)

* clippy

* formatting

* chore(ci): Use correct rust version

* review comment changes

* Accepted suggestion from @phated

* clippy
```

The person merging would remove extrenous messaging and keep only the relavent Conventional Commits:

```
feat(syntax): Implement String data type (#123)

chore(ci): Use correct rust version
```

Additional Conventional Commits can be added before squashing if they improve the Changelog or commit history:

```
feat(syntax): Implement String data type (#123)

chore(ci): Use correct rust version
chore(clippy): Update all println calls to use single argument syntax
```

### Merge Checklist

Before merging, you should mentally review these questions:

- Is continuous integration passing?
- Do you have the required amount of approvals?
- Does anyone else need to be pinged for thoughts?
- Will this cause problems for our release schedule? For example: maybe a patch release still needs to be published.
- What details do we want to convey to users in the Changelog?

## Releases

Noir releases are managed by [Release Please](https://github.com/googleapis/release-please) which runs in a GitHub Action whenever a commit is made on the `master` branch.

Release Please parses Conventional Commit messages and opens (or updates) a pull request against the `master` branch that contains updates to the versions & Changelog within the project. If it doesn't detect any breaking change commits, it will only increment the "patch" version; however, if it detects a breaking change commit, it will increment the "minor" version number to indicate a breaking release.

When we are ready to release the version, we approve and squash the release pull request into `master`. Release Please will detect this merge and generate the appropriate tags for the release. Additional release steps may be triggered inside the GitHub Action to automate other parts of the release process.

## Changelog

Noir's Changelog is automatically managed by Release Please and informed by the Conventional Commits (as discussed above).

Given the following commits:
- `feat(syntax): Implement String data type (#123)`
- `chore(ci): Use correct rust version`
- `fix(optimizer): Compile Boolean to u1`
- `feat(nargo): Add support for wasm backend (#234)`
- `feat!: Rename nargo build to nargo check (#693)`
- `feat(nargo)!: Enforce minimum rustc version`

Release Please would generate add the following to the Changelog:

```markdown
## [0.2.0](https://github.com/noir-lang/noir/compare/noir-v0.1.0...noir-v0.2.0) (2023-01-25)

### ⚠ BREAKING CHANGES

* **nargo:** Enforce minimum rustc version
* Rename nargo build to nargo check (#693)

### Features

* **nargo:** Enforce minimum rustc version
* Rename nargo build to nargo check (#693)
* **nargo:** Add support for wasm backend (#234)
* **syntax:** Implement String data type (#123)

### Bug Fixes

* **optimizer:** Compile Boolean to u1
```
