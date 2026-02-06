---
name: extract-fuzzer-repro
description: Extract a Noir reproduction project from fuzzer failure logs in GitHub Actions. Use when a CI fuzzer test fails and you need to create a local reproduction.
allowed-tools: Bash, Read, Write
---

# Extract Fuzzer Reproduction from GitHub Actions

Use this skill to extract a reproduction project from fuzzer failures in CI. This creates a minimal Noir project you can use locally to reproduce and debug the issue.

## 1. Get the Job ID

From the GitHub Actions URL, extract the job ID. The URL format is:
```
https://github.com/noir-lang/noir/actions/runs/RUN_ID/job/JOB_ID
```

## 2. Fetch the Logs

```bash
gh api repos/noir-lang/noir/actions/jobs/JOB_ID/logs 2>&1 | tee fuzzer_logs.txt
```

## 3. Find the Reproduction Code

Search for the generated Noir source:
```bash
grep -nE "unconstrained fn main|fn main" fuzzer_logs.txt
```

The fuzzer output has this structure:
```
---
AST:
global G_A: i8 = -127_i8;
unconstrained fn main(a: pub i8) -> pub i8 {
    ...
}
---
ABI Inputs:
a = "-0x5c"
---
Seed: 0xc63ed07b00100000
```

## 4. Check for Required Compiler Flags

Some failures only manifest with specific flags (e.g., `-Zenums` for match expressions, optimization flags, etc.). Note any flags mentioned in the logs as you'll need them to reproduce the issue.

## 5. Create the Reproduction Project

```bash
nargo new repro_project
cd repro_project
```

Copy the AST section to `src/main.nr` and create `Prover.toml` with the ABI inputs.

## 6. Verify the Reproduction

```bash
# Include any required flags from the logs
nargo execute
```

This should reproduce the failure you saw in CI. If it doesn't, double-check you're using the same compiler flags from the logs.
