import {
  depsScriptSourcePath,
  depsScriptExpectedArtifact,
  libASourcePath,
  libBSourcePath,
  simpleScriptSourcePath,
  simpleScriptExpectedArtifact,
} from '../../shared';
import { readFileSync } from 'fs';
import { resolve } from 'path';

import { compile, PathToFileSourceMap } from '../../../build/cjs';

async function getPrecompiledSource(path: string): Promise<any> {
  const compiledData = readFileSync(resolve(import.meta.url, path)).toString();
  return JSON.parse(compiledData);
}

describe('noir wasm compilation', () => {
  describe('can compile simple scripts', () => {
    it('matching nargos compilation', async () => {
      const sourceMap = new PathToFileSourceMap();
      sourceMap.add_source_code(simpleScriptSourcePath, readFileSync(simpleScriptSourcePath, 'utf-8'));
      const wasmCircuit = compile(simpleScriptSourcePath, undefined, undefined, sourceMap);
      const cliCircuit = await getPrecompiledSource(simpleScriptExpectedArtifact);

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.program.bytecode).toEqual(cliCircuit.bytecode);
      expect(wasmCircuit.program.abi).toEqual(cliCircuit.abi);
      expect(wasmCircuit.program.backend).toEqual(cliCircuit.backend);
    });
  });

  describe('can compile scripts with dependencies', () => {
    const sourceMap = new PathToFileSourceMap();
    beforeEach(() => {
      sourceMap.add_source_code('script/main.nr', readFileSync(depsScriptSourcePath, 'utf-8'));
      sourceMap.add_source_code('lib_a/lib.nr', readFileSync(libASourcePath, 'utf-8'));
      sourceMap.add_source_code('lib_b/lib.nr', readFileSync(libBSourcePath, 'utf-8'));
    });

    it('matching nargos compilation', async () => {
      const wasmCircuit = compile(
        'script/main.nr',
        false,
        {
          root_dependencies: ['lib_a'],
          library_dependencies: {
            lib_a: ['lib_b'],
          },
        },
        sourceMap,
      );

      const cliCircuit = await getPrecompiledSource(depsScriptExpectedArtifact);

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.program.bytecode).toEqual(cliCircuit.bytecode);
      expect(wasmCircuit.program.abi).toEqual(cliCircuit.abi);
      expect(wasmCircuit.program.backend).toEqual(cliCircuit.backend);
    });
  });
});
