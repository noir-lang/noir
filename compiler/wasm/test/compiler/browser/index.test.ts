import { getPaths } from '../../shared';
import { expect } from '@esm-bundle/chai';

// @ts-expect-error Import without typings, probably there's a better way
import { compile, createFileManager } from '../../../dist/web/main';
import { FileManager } from '../../../dist/types/noir/file-manager/file-manager';
import { NoirCompiledContract } from '../../../dist/types/types/noir_artifact';

const paths = getPaths('.');

async function getFile(path: string) {
  const basePath = new URL('./../../../', import.meta.url).toString().replace(/\/$/g, '');
  const url = `${basePath}${path.replace('.', '')}`;
  const response = await fetch(url);
  return response;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(path: string): Promise<any> {
  const response = await getFile(path);
  const compiledData = await response.text();
  return JSON.parse(compiledData);
}

describe('noir-compiler', () => {
  it('both nargo and noir_wasm should compile identically', async () => {
    const { contractExpectedArtifact } = paths;
    const fm: FileManager = createFileManager('/');
    const files = Object.values(paths).filter((fileOrDir) => /^\.?\/.*\..*$/.test(fileOrDir));
    for (const path of files) {
      await fm.writeFile(path, (await getFile(path)).body as ReadableStream<Uint8Array>);
    }
    const nargoArtifact = (await getPrecompiledSource(contractExpectedArtifact)) as NoirCompiledContract;
    nargoArtifact.functions.sort((a, b) => a.name.localeCompare(b.name));
    const noirWasmArtifact = await compile(fm, '/public/fixtures/noir-contract');
    const noirWasmContract = noirWasmArtifact[0].contract as NoirCompiledContract;
    expect(noirWasmContract).not.to.be.undefined;
    noirWasmContract.functions.sort((a, b) => a.name.localeCompare(b.name));
    expect(nargoArtifact).to.deep.eq(noirWasmContract);
  });
});
