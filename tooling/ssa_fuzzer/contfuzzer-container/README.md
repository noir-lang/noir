# SSA Fuzzer — ContFuzzer Container

ContFuzzer-compatible container for the `acir_vs_brillig` fuzz target.

Differentially tests ACIR vs Brillig execution of randomly generated SSA
programs. When the two runtimes produce different results, the fuzzer
crashes — ContFuzzer ingests the crash as a finding.

## Build

```bash
docker build -t contfuzzer-ssa-fuzzer .

# Pin to a specific noir commit:
docker build --build-arg COMMIT=abc123 -t contfuzzer-ssa-fuzzer .
```

## Run standalone (platform-like flags)

```bash
mkdir -p /tmp/{corpus,crashes,output}

docker run --rm --read-only --tmpfs /tmp:size=512m \
    --user 65534:10001 \
    -v /tmp/corpus:/corpus \
    -v /tmp/crashes:/crashes \
    -v /tmp/output:/output \
    -e FUZZ_MODE=fuzz \
    -e FUZZ_TARGET=acir_vs_brillig \
    -e FUZZ_TIMEOUT=60 \
    -e FUZZ_JOBS=4 \
    contfuzzer-ssa-fuzzer
```

## Register with ContFuzzer

```bash
curl -X POST http://localhost:8000/api/{org}/{project}/fuzzers/ingest \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"image_digest\": \"contfuzzer-ssa-fuzzer:latest\",
    \"repo\": \"noir-lang/noir\",
    \"branch\": \"master\",
    \"manifest\": $(cat fuzzer_manifest.json)
  }"
```

## What it does NOT include

- **Redis integration** — the upstream `acir_vs_brillig` target optionally
  pushes to Redis via `REDIS_URL`. This container does not set that variable,
  so the fuzzer runs standalone without Redis.
- **Triage mode** — the upstream `TRIAGE` env var is not used. Triage SSA
  pass output by re-running a crash locally with `TRIAGE=FULL`.
- **`brillig` target** — requires a Node.js simulator and transpiler binary
  at runtime. Not compatible with ContFuzzer's read-only container contract.
