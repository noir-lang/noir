# Benchmarks

Tests in this folder are meant to used for benchmarking. Stats are collected by instrumenting relevant sections of the code and emitting stats via structured logging. All stats are strongly-typed and defined in `yarn-project/types/src/stats/stats.ts`.

These stats are emitted to jsonl files named after the test being run if the `BENCHMARK` flag is enabled or if running on `CI`. This setup happens when calling the `setup` helper of e2e tests in `yarn-project/end-to-end/src/fixtures/logging.ts`. Note that by default stats from all e2e tests are collected on the CI, and are uploaded to S3 using the `upload_logs_to_s3.sh` script called at the end of `run_tests_local`. All jsonl files are uploaded to the `aztec-ci-artifacts` bucket under the `logs` folder. Tests run in master are uploaded to `logs/master/COMMIT_HASH`, while tests from a PR are uploaded to `logs/pulls/PULL_REQUEST_NUMBER`.

## Benchmark summaries

After all benchmark tests are executed, a `bench-summary` CI job takes care of aggregating them, using the scripts in `yarn-project/scripts/benchmarks` orchestrated by `scripts/ci/assemble_e2e_benchmark.sh`. This script downloads all jsonl files, extracts metrics grouped by block size or chain length, and outputs an aggregated benchmark json file which is uploaded to S3. This file is uploaded to the same `aztec-ci-artifacts` bucket but under the `benchmarks` folder.

Metrics are strongly typed as well and defined in `yarn-project/types/src/stats/metrics.ts`, while the `yarn-project/scripts/src/benchmarks/aggregate.ts` script takes care of generating them out of the collected stats from the jsonl files.

Once the summary is generated, if the benchmark run is on a PR, then the summary job will also download the latest benchmark from master, compare it against the current run, generate a markdown summary, and post it to the pull request on github. This uses the `AZTEC_BOT_COMMENTER_GITHUB_TOKEN`, which is a fine-grained personal access token from the `AztecBot` github user with rw permissions on issues and pull requests.

## Local development

To test locally, first run one or more benchmark e2e tests locally from the `yarn-project/end-to-end` folder. It may be a good idea to shorten them so they run faster, for example:

```
$ BENCHMARK_BLOCK_SIZES=4 BENCHMARK=1 yarn test bench_publish
```

This should've generated one or more jsonl files in `yarn-project/end-to-end/log`
