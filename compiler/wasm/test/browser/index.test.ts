import { expect } from '@esm-bundle/chai';
import initNoirWasm from '@noir-lang/noir_wasm';
import { compileNoirSource, nargoArtifactPath, noirSourcePath } from '../shared';

beforeEach(async () => {
  await initNoirWasm();
});

async function getFileContent(path: string): Promise<string> {
  const mainnrSourceURL = new URL(path, import.meta.url);
  const response = await fetch(mainnrSourceURL);
  return await response.text();
}

async function getSource(): Promise<string> {
  return getFileContent(noirSourcePath);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(): Promise<any> {
  const compiledData = await getFileContent(nargoArtifactPath);
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  it('matches nargos compilation', async () => {
    const source = await getSource();

    const wasmCircuit = await compileNoirSource(source);

    const cliCircuit = await getPrecompiledSource();

    // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
    expect(wasmCircuit.bytecode).to.eq(cliCircuit.bytecode);
    expect(wasmCircuit.abi).to.deep.eq(cliCircuit.abi);
    expect(wasmCircuit.backend).to.eq(cliCircuit.backend);
  }).timeout(20e3); // 20 seconds
});
