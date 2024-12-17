# Critical Libraries Status

This directory contains one `.failures.jsonl` file per external directory that is checked by CI.
CI will run the external repository tests and compare the test failures against those recorded
in these files. If there's a difference, CI will fail.

This allows us to mark some tests as expected to fail if we introduce breaking changes.
When tests are fixed on the external repository, CI will let us know that we need to remove
the `.failures.jsonl` failures on our side.

The format of the `.failures.jsonl` files is one JSON per line with a failure:

```json
{"suite":"one","name":"foo"}
```

If it's expected that an external repository doesn't compile (because a PR introduces breaking changes
to, say, the type system) you can remove the `.failures.jsonl` file for that repository and CI
will pass again. Once the repository compiles again, CI will let us know and require us to put
back the `.failures.jsonl` file.