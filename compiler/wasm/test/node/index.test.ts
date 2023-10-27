import { expect } from 'chai';
import {
  depsScriptSourcePath,
  depsScriptExpectedArtifact,
  libASourcePath,
  libBSourcePath,
  simpleScriptSourcePath,
  simpleScriptExpectedArtifact,
} from '../shared';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { compile } from '@noir-lang/noir_wasm';
import { initializeResolver } from '@noir-lang/source-resolver';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(path: string): Promise<any> {
  const compiledData = readFileSync(resolve(__dirname, path)).toString();
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  describe('can compile simple scripts', () => {
    it('matching nargos compilation', async () => {
      const wasmCircuit = await compile(join(__dirname, simpleScriptSourcePath));
      const cliCircuit = await getPrecompiledSource(simpleScriptExpectedArtifact);

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.bytecode).to.eq(cliCircuit.bytecode);
      expect(wasmCircuit.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.backend).to.eq(cliCircuit.backend);
    }).timeout(10e3);
  });

  describe('can compile scripts with dependencies', () => {
    beforeEach(() => {
      // this test requires a custom resolver in order to correctly resolve dependencies
      initializeResolver((file) => {
        switch (file) {
          case '/script/main.nr':
            return readFileSync(join(__dirname, depsScriptSourcePath), 'utf-8');

          case '/lib_a/lib.nr':
            return readFileSync(join(__dirname, libASourcePath), 'utf-8');

          case '/lib_b/lib.nr':
            return readFileSync(join(__dirname, libBSourcePath), 'utf-8');

          default:
            return '';
        }
      });
    });

    it('matching nargos compilation', async () => {
      const wasmCircuit = await compile('/script/main.nr', false, {
        root_dependencies: ['lib_a'],
        library_dependencies: {
          lib_a: ['lib_b'],
        },
      });

      const cliCircuit = await getPrecompiledSource(depsScriptExpectedArtifact);

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.bytecode).to.eq(cliCircuit.bytecode);
      expect(wasmCircuit.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.backend).to.eq(cliCircuit.backend);
    }).timeout(10e3);
  });
});
