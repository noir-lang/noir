---
title: VS Code Debugger
description:
  VS Code Debugger configuration and features.
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Debugger,
    VS Code,
    IDE,
  ]
sidebar_position: 0
---

# VS Code Noir Debugger Reference

The Noir debugger enabled by the vscode-noir extension ships with default settings such that the most common scenario should run without any additional configuration steps.

These defaults can be overridden by defining a launch configuration though. This page provides a reference for the properties you can override via launch config, as well as documenting the Nargo `dap` command, which is a depedency of the VS Code Noir debugger. 


## Creating and editing launch configuration files

To create a launch configuration file from VS Code, open the _debug pane_, and click on _create a launch.json file_. 

ref1-create-launch

A `launch.json` file will be created, populated with basic defaults. 

### Noir Debugger launch.json properties

#### `projectFolder`

Optional. String. Absolute path to the Nargo project to debug. By default, it is dynamically determined by looking for the nearest Nargo.toml file to the active file at the moment of launching the debugger. 

#### `proverName`

Optional. String. Name of the prover input to use. Defaults to `Prover`, which looks for a file named `Prover.toml` at the `projectFolder`.

#### generateAcir

Optional. Boolean. If true, generate ACIR opcodes instead of Brillig which will be closer to release binaries but less convenient for debugging. Defaults to false.
                
#### skipInstrumentation

Optional. Boolean. Skips variables debugging instrumentation of code, making debugging less convenient but the resulting binary smaller and closer to production. Defaults to false.

Note: skipping instrumentation causes the debugger to be unable to inspect local variables.

## `nargo dap [OPTIONS]`

When run without any option flags, it starts the Nargo Debug Adapter Protocol server, which acts as the debugging backend for the VS Code Noir Debugger. 

All option flags are related to preflight checks. The Debug Adapter Protocol specifies how errors are to be informed from a running DAP server, but doesn't specify mechanisms to communicate server initialization errors between the DAP server and its client IDE. 

Thus `nargo dap` ships with a _preflight check_ mode. If flag `--preflight-check` and the rest of the required `--preflight-*` flags are provided, Nargo will run the same initialization routine except starting the DAP server.

`vscode-noir` will then run `nargo dap` in preflight check mode first before a debugging session starts. If the preflight check ends in error, vscode-noir will present output from this process in its own Output pane in VS Code. This makes it possible for user to diagnose what pieces of configuration might be wrong or missing.

If the preflight check succeeds, `vscode-noir` proceeds to start the DAP server normally buy running `nargo dap` without any additional flags.

### Options

| Option                                  | Description                                                                         |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `--preflight-check`                     | If present, dap runs in preflight check mode.                               |
| `--preflight-project-folder <PREFLIGHT_PROJECT_FOLDER>`   | Absolute path to the project to debug for preflight check.                        |
| `--preflight-prover-name <PREFLIGHT_PROVER_NAME>`       | Name of prover file to use for preflight check                              |
| `--preflight-generate-acir`                 | Optional. If present, compile in ACIR mode while running preflight check.                                 |
| `--preflight-skip-instrumentation`            | Optional. If present, compile without introducing debug instrumentation while running preflight check.  |
| `-h, --help`                            | Print help.                                               |
