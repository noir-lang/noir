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

  getSubdirs(join(testProgramsDir, 'compile_success_empty')).forEach((name: string) => {
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

  getSubdirs(join(testProgramsDir, 'execution_success')).forEach((name: string) => {
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
