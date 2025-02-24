import { join, resolve } from 'path';
import { getPaths } from '../../shared';

import { expect } from 'chai';
import { compile_program, compile_contract, createFileManager } from '@noir-lang/noir_wasm';
import { readFile } from 'fs/promises';
import { ContractArtifact, ProgramArtifact } from '../../../src/types/noir_artifact';
import { shouldCompileContractIdentically, shouldCompileProgramIdentically } from '../shared/compile.test';

const basePath = resolve(join(__dirname, '../../'));

describe('noir-compiler/node', () => {
  shouldCompileProgramIdentically(
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
});
