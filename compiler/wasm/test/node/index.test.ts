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
import { compile, PathToFileSourceMap } from '@noir-lang/noir_wasm';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(path: string): Promise<any> {
  const compiledData = readFileSync(resolve(__dirname, path)).toString();
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  describe('can compile simple scripts', () => {
    it('matching nargos compilation', async () => {
      const sourceFileMap = new PathToFileSourceMap();
      sourceFileMap.add_source_code(
        join(__dirname, simpleScriptSourcePath),
        readFileSync(join(__dirname, simpleScriptSourcePath), 'utf-8'),
      );
      const wasmCircuit = await compile(join(__dirname, simpleScriptSourcePath), undefined, undefined, sourceFileMap);
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
    let map: PathToFileSourceMap;
    beforeEach(() => {
      // this test requires a custom resolver in order to correctly resolve dependencies

      map = new PathToFileSourceMap();
      map.add_source_code('script/main.nr', readFileSync(join(__dirname, depsScriptSourcePath), 'utf-8'));
      map.add_source_code('lib_a/lib.nr', readFileSync(join(__dirname, libASourcePath), 'utf-8'));
      map.add_source_code('lib_b/lib.nr', readFileSync(join(__dirname, libBSourcePath), 'utf-8'));
    });

    it('matching nargos compilation', async () => {
      const wasmCircuit = await compile(
        'script/main.nr',
        false,
        {
          root_dependencies: ['lib_a'],
          library_dependencies: {
            lib_a: ['lib_b'],
          },
        },
        map,
      );

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
