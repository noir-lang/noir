# Lessons Learned

## Always assume the test suite passes on master

- **Failure mode**: Attempted to `git stash` and re-run tests on master to check if failures were pre-existing, wasting time and requiring user intervention.
- **Detection signal**: When branched off master and seeing test failures, the instinct was to verify against master rather than debug the regression.
- **Prevention rule**: If branched off of master, always assume the full test suite is passing. Any test failure is a regression caused by the current changes. Focus on debugging why the change broke existing tests rather than checking master.
