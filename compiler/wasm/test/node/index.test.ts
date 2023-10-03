import { expect } from 'chai';
import { compileNoirSource, nargoArtifactPath, noirSourcePath } from '../shared';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

async function getFileContent(path: string): Promise<string> {
  return readFileSync(join(__dirname, path)).toString();
}

async function getSource(): Promise<string> {
  return getFileContent(noirSourcePath);
}

async function getPrecompiledSource(): Promise<string> {
  const compiledData = await getFileContent(nargoArtifactPath);
  return JSON.parse(compiledData).bytecode;
}

describe('noir wasm compilation', () => {
  it('matches nargos compilation', async () => {
    const source = await getSource();

    const wasmCircuitBase64 = await compileNoirSource(source);

    const cliCircuitBase64 = await getPrecompiledSource();

    console.log('wasm', wasmCircuitBase64);

    console.log('cli', cliCircuitBase64);

    console.log('Compilation is a match? ', wasmCircuitBase64 === cliCircuitBase64);

    expect(wasmCircuitBase64).to.equal(cliCircuitBase64);
  }).timeout(10e3);
});
