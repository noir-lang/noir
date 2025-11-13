import { join, resolve } from 'path';
import { getPaths } from '../../shared';

import { expect } from 'chai';
import { compile_program, compile_contract, createFileManager } from '@noir-lang/noir_wasm';
import { readFile } from 'fs/promises';
import { ContractArtifact, ProgramArtifact } from '../../../src/types/noir_artifact';
import {
  shouldCompileContractIdentically,
  shouldCompileProgramIdentically,
  shouldCompileProgramSuccessfully,
} from '../shared/compile.test';
import { readdirSync } from 'fs';

const testProgramsDir = resolve(join(__dirname, '../../../../../test_programs'));
const basePath = resolve(join(__dirname, '../../'));

function getSubdirs(path: string): string[] {
  return readdirSync(path, { withFileTypes: true })
    .filter((dirent) => dirent.isDirectory())
    .map((dirent) => dirent.name);
}

describe('noir-compiler/node', () => {
  shouldCompileProgramIdentically(
    'simple',
    async () => {
      const { simpleScriptProjectPath, simpleScriptExpectedArtifact } = getPaths(basePath);

      const fm = createFileManager(simpleScriptProjectPath);
      const nargoArtifact = JSON.parse((await readFile(simpleScriptExpectedArtifact)).toString()) as ProgramArtifact;
      const noirWasmArtifact = await compile_program(fm);
      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    /*30 second timeout*/ 30000,
  );

  shouldCompileProgramIdentically(
    'deps',
    async () => {
      const { depsScriptProjectPath, depsScriptExpectedArtifact } = getPaths(basePath);

      const fm = createFileManager(depsScriptProjectPath);
      const nargoArtifact = JSON.parse((await readFile(depsScriptExpectedArtifact)).toString()) as ProgramArtifact;
      const noirWasmArtifact = await compile_program(fm);
      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    /*30 second timeout*/ 30000,
  );

  shouldCompileContractIdentically(
    'noir-contract',
    async () => {
      const { contractProjectPath, contractExpectedArtifact } = getPaths(basePath);

      const fm = createFileManager(contractProjectPath);
      const nargoArtifact = JSON.parse((await readFile(contractExpectedArtifact)).toString()) as ContractArtifact;
      const noirWasmArtifact = await compile_contract(fm);
      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    /*30 second timeout*/ 30000,
  );

  const filteredCompileSuccessEmptyTests = [
    'comptime_enums',
    'enums',
    'regression_7570_nested',
    'regression_7570_serial',
    'workspace_reexport_bug',
    'overlapping_dep_and_mod',
  ];
  getSubdirs(join(testProgramsDir, 'compile_success_empty'))
    .filter((name) => !filteredCompileSuccessEmptyTests.includes(name))
    .forEach((name: string) => {
      shouldCompileProgramSuccessfully(
        name,
        async () => {
          const programDir = join(testProgramsDir, 'compile_success_empty', name);
          const fm = createFileManager(programDir);
          const noirWasmArtifact = await compile_program(
            fm,
            undefined,
            (_) => {},
            (_) => {},
          );
          return noirWasmArtifact;
        },
        expect,
        /*30 second timeout*/ 30000,
      );
    });

  const filteredExecutionSuccessTests = [
    'custom_entry',
    'overlapping_dep_and_mod',
    'regression_7323',
    'workspace',
    'workspace_default_member',
    'regression_9294', // TODO: Requires the 'enums' unstable feature.
  ];
  getSubdirs(join(testProgramsDir, 'execution_success'))
    .filter((name) => !filteredExecutionSuccessTests.includes(name))
    .forEach((name: string) => {
      shouldCompileProgramSuccessfully(
        name,
        async () => {
          const programDir = join(testProgramsDir, 'execution_success', name);
          const fm = createFileManager(programDir);
          const noirWasmArtifact = await compile_program(
            fm,
            undefined,
            (_) => {},
            (_) => {},
          );
          return noirWasmArtifact;
        },
        expect,
        /*30 second timeout*/ 30000,
      );
    });
});
