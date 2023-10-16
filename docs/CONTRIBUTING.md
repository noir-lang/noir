# Contributing to Noir

Thank you for your interest in contributing to Noir documentation! We value your contributions in making Noir better.

This guide will discuss how the Noir team handles [Commits](#commits), [Pull Requests](#pull-requests), [Merging](#merging), and [Versioning](#versioning).

__Note:__ We won't force external contributors to follow this verbatim, but following these guidelines definitely helps us in accepting your contributions.

## Commits

We want to keep our commits small and focused. This allows for easily reviewing individual commits and/or splitting up pull requests when they grow too big. Additionally, this allows us to merge smaller changes quicker and release more often.

When committing, it's often useful to use the `git add -p` workflow to decide on what parts of the changeset to stage for commit.

We don't currently enforce any commit standard, however that may change at any time. Mind that the [Noir](https://github.com/noir-lang/noir) repo does enforce the [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0/) standard.

## Pull Requests

Before you create a pull request, search for any issues related to the change you are making. If none exist already, create an issue that thoroughly describes the problem that you are trying to solve. These are used to inform reviewers of the original intent and should be referenced via the pull request template.

Pull Requests should be focused on the specific change they are working towards. If prerequisite work is required to complete the original pull request, that work should be submitted as a separate pull request.

This strategy avoids scenarios where pull requests grow too large/out-of-scope and don't get proper reviews—we want to avoid "LGTM, I trust you" reviews.

The easiest way to do this is to have multiple commits while you work and then you can cherry-pick the smaller changes into separate branches for pull requesting.

### Reviews

For any repository in the noir-lang organization, we require code review & approval by __one__ Noir team member before the changes are merged. However, while the docs repository is still getting up-to-speed with the current Noir fetures, we do allow for non-breaking pull requests to be merged at any time. Breaking pull requests should only be merged when the team has general agreement of the changes.

The CI/CD workflow at Netlify should provide you with a preview of the website once merged. Use this preview to thoroughly test the changes before requesting reviews or merging.

## Merging

Once approved by the required number of team members, the pull request can be merged into the `master` branch. Sometimes, especially for external contributions, the final approver may merge the pull request instead of the submitter.

### Merge Checklist

Before merging, you should mentally review these questions:

- Is continuous integration passing?
- Do you have the required amount of approvals?
- Does anyone else need to be pinged for thoughts?

## Versioning

The Noir documentation is versioned according to the [Docusaurus documentation](https://docusaurus.io/docs/versioning). In the `versioned_docs` and `versioned_sidebar` folders you will find the docs and configs for the previous versions. If any change needs to be made to older versions, please do it in this folder.

In the `docs` folder, you'll find the current, unreleased version, which we call `dev`. Any change in this folder will be reflected in the next version, once the Noir team decides to release.

We aim to have every version matching the versions of [Noir](https://github.com/noir-lang/noir). However, we would only cut a new version of the docs if there are breaking or otherwise significant changes, to avoid unecessary build time and size to the existent documentation.

While the versioning is intended to be managed by the core maintainers, we feel it's important for external contributors to understand why and how is it maintained. To bump to a new version, run the following command, replacing with the intended version:

`npm run docusaurus docs:version <new_version_tag>`

This should create a new version by copying the `docs` folder and the `sidebars.js` file to the relevant folders, as well as adding this version to `versions.json`.
