import { join, resolve } from 'path';
import { getPaths } from '../../shared';

import { expect } from 'chai';
import { readFile } from 'fs/promises';
import { compile, createFileManager } from '@noir-lang/noir_wasm';
import { CompiledContract } from '../../../src/types/noir_artifact';

const basePath = resolve(join(__dirname, '../../'));
const { contractProjectPath, contractExpectedArtifact } = getPaths(basePath);

describe('noir-compiler', () => {
  it('both nargo and noir_wasm should compile identically', async () => {
    const fm = createFileManager(contractProjectPath);
    const nargoArtifact = JSON.parse((await readFile(contractExpectedArtifact)).toString()) as CompiledContract;
    nargoArtifact.functions.sort((a, b) => a.name.localeCompare(b.name));
    const noirWasmArtifact = await compile(fm);
    if (!('contract' in noirWasmArtifact)) {
      throw new Error('Compilation failed');
    }
    const noirWasmContract = noirWasmArtifact.contract;
    expect(noirWasmContract).not.to.be.undefined;
    noirWasmContract.functions.sort((a, b) => a.name.localeCompare(b.name));
    expect(nargoArtifact).to.deep.eq(noirWasmContract);
  });
});
