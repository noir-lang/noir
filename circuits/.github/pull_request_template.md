# Description

Please provide a paragraph or two giving a summary of the change, including relevant motivation and context.

# Checklist:

- [ ] I have reviewed my diff in github, line by line.
- [ ] Every change is related to the PR description.
- [ ] I have [linked](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue) this pull request to the issue(s) that it resolves.
- [ ] There are no unexpected formatting changes, superfluous debug logs, or commented-out code.
- [ ] There are no circuit changes, OR specifications in `/specs` have been updated.
- [ ] There are no circuit changes, OR a cryptographer has been assigned for review.
- [ ] I've updated any terraform that needs updating (e.g. environment variables) for deployment.
- [ ] The branch has been rebased against the head of its merge target.
- [ ] I'm happy for the PR to be merged at the reviewer's next convenience.
- [ ] New functions, classes, etc. have been documented according to the doxygen comment format. Classes and structs must have `@brief` describing the intended functionality.
- [ ] If existing code has been modified, such documentation has been added or updated.

> **Note**
> If you are updating the submodule, please make sure you do it in its own _special_ PR and avoid making changes to the submodule as a part of other PRs.
> To update a submodule, you can run the following commands:
> ```console
> $ git submodule update --recursive
> ```
> Alternatively, you can select a particular commit in `barretenberg/aztec3` that you wish to point to:
> ```console
> $ cd barretenberg
> $ git pull origin aztec3        # This will point to the latest commit in `barretenberg/aztec3`
> $ git checkout <commit_hash>    # Use this if you wish to point to a particular commit.
> $ cd ..
> $ git add . && git commit -m <commit_msg>
> $ git push
> ```
