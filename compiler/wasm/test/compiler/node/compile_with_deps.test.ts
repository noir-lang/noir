import { join, resolve } from 'path';
import { getPaths } from '../../shared';

import { expect } from 'chai';
import { compile, createFileManager } from '@noir-lang/noir_wasm';
import { readFile } from 'fs/promises';
import { ContractArtifact } from '../../../src/types/noir_artifact';
import { shouldCompileIdentically } from '../shared/compile_with_deps.test';

const basePath = resolve(join(__dirname, '../../'));
const { contractProjectPath, contractExpectedArtifact } = getPaths(basePath);

describe('noir-compiler/node', () => {
  shouldCompileIdentically(async () => {
    const fm = createFileManager(contractProjectPath);
    const nargoArtifact = JSON.parse((await readFile(contractExpectedArtifact)).toString()) as ContractArtifact;
    const noirWasmArtifact = await compile(fm);
    return { nargoArtifact, noirWasmArtifact };
  }, expect);
});
