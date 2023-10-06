import { expect } from 'chai';
import { nargoArtifactPath, noirSourcePath } from '../shared';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { compile } from '@noir-lang/noir_wasm';

async function getFileContent(path: string): Promise<string> {
  return readFileSync(join(__dirname, path)).toString();
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(): Promise<any> {
  const compiledData = await getFileContent(nargoArtifactPath);
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  it('matches nargos compilation', async () => {
    const wasmCircuit = await compile(noirSourcePath);
    const cliCircuit = await getPrecompiledSource();

    // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
    expect(wasmCircuit.bytecode).to.eq(cliCircuit.bytecode);
    expect(wasmCircuit.abi).to.deep.eq(cliCircuit.abi);
    expect(wasmCircuit.backend).to.eq(cliCircuit.backend);
  }).timeout(10e3);
});
