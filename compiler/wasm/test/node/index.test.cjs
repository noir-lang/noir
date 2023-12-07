/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable no-undef */
const { expect } = require('chai');
const { readFileSync } = require('fs');
const { join, resolve } = require('path');
const { initializeResolver } = require('@noir-lang/source-resolver');
const shared = require('../shared.cjs');

const { compile } = require('../../dist/node/main.js');

const {
  depsScriptSourcePath,
  depsScriptExpectedArtifact,
  libASourcePath,
  libBSourcePath,
  simpleScriptSourcePath,
  simpleScriptExpectedArtifact,
} = shared;

const { describe, it } = require('mocha');

async function getPrecompiledSource(path) {
  const compiledData = readFileSync(resolve(__dirname, path)).toString();
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  describe('can compile simple scripts', () => {
    it('matching nargos compilation', async () => {
      const wasmCircuit = await compile(join(__dirname, simpleScriptSourcePath));
      const cliCircuit = await getPrecompiledSource(simpleScriptExpectedArtifact);

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
      expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.program.backend).to.eq(cliCircuit.backend);
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

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
      expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.program.backend).to.eq(cliCircuit.backend);
    }).timeout(10e3);
  });
});
