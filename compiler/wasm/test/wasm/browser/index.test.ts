import {
  depsScriptExpectedArtifact,
  depsScriptSourcePath,
  libASourcePath,
  libBSourcePath,
  simpleScriptExpectedArtifact,
  simpleScriptSourcePath,
} from '../../shared';
import esmBuild from '../../../build/esm';
import { expect } from 'chai';

const { PathToFileSourceMap, compile } = esmBuild;

async function getFileContent(path: string): Promise<string> {
  // @ts-expect-error import.meta.url is only defined in the context of ESM, in which this test runs
  const url = new URL(path, import.meta.url);
  const response = await fetch(url);
  return await response.text();
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getPrecompiledSource(path: string): Promise<any> {
  const compiledData = await getFileContent(path);
  return JSON.parse(compiledData);
}

describe('noir wasm', () => {
  describe('can compile script without dependencies', () => {
    it('matching nargos compilation', async () => {
      const sourceMap = new PathToFileSourceMap();
      sourceMap.add_source_code('main.nr', await getFileContent(simpleScriptSourcePath));

      const wasmCircuit = compile('main.nr', undefined, undefined, sourceMap);
      const cliCircuit = await getPrecompiledSource(simpleScriptExpectedArtifact);

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
      expect(wasmCircuit.program.noir_version).to.eq(cliCircuit.noir_version);
      expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
    });
  });

  describe('can compile script with dependencies', () => {
    it('matching nargos compilation', async () => {
      const [scriptSource, libASource, libBSource] = await Promise.all([
        getFileContent(depsScriptSourcePath),
        getFileContent(libASourcePath),
        getFileContent(libBSourcePath),
      ]);

      const sourceMap = new PathToFileSourceMap();
      sourceMap.add_source_code('script/main.nr', scriptSource);
      sourceMap.add_source_code('lib_a/lib.nr', libASource);
      sourceMap.add_source_code('lib_b/lib.nr', libBSource);

      const wasmCircuit = await compile(
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

      if (!('program' in wasmCircuit)) {
        throw Error('Expected program to be present');
      }

      const cliCircuit = await getPrecompiledSource(depsScriptExpectedArtifact);

      // We don't expect the hashes to match due to how `noir_wasm` handles dependencies

      expect(wasmCircuit.program.noir_version).to.eq(cliCircuit.noir_version);
      expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
      expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
    });
  });
});
