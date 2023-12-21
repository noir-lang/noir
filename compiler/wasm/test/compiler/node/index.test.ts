import { join, resolve } from 'path';
import { getPaths } from '../../shared';

import { expect } from 'chai';
import { readFile } from 'fs/promises';

import type { NoirCompiledContract } from '../../../dist/types/types/noir_artifact';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const cjs = require('../../../dist/node/main');
const { createFileManager, compile } = cjs;

const basePath = resolve(join(__dirname, '../../../'));
const { contractProjectPath, contractExpectedArtifact } = getPaths(basePath);

describe('noir-compiler', () => {
  it('both nargo and noir_wasm should compile identically', async () => {
    const fm = createFileManager(contractProjectPath);
    const nargoArtifact = JSON.parse((await readFile(contractExpectedArtifact)).toString()) as NoirCompiledContract;
    nargoArtifact.functions.sort((a, b) => a.name.localeCompare(b.name));
    const noirWasmArtifact = await compile(fm);
    const noirWasmContract = noirWasmArtifact[0].contract as NoirCompiledContract;
    expect(noirWasmContract).not.to.be.undefined;
    noirWasmContract.functions.sort((a, b) => a.name.localeCompare(b.name));
    expect(nargoArtifact).to.deep.eq(noirWasmContract);
  });
});
