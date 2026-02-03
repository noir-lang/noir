/* eslint-disable @typescript-eslint/ban-ts-comment */
import { getPaths } from '../../shared';
import { expect } from '@esm-bundle/chai';
import { compile_program, compile_contract, createFileManager } from '@noir-lang/noir_wasm';
import { ContractArtifact, ProgramArtifact } from '../../../src/types/noir_artifact';
import { shouldCompileContractIdentically, shouldCompileProgramIdentically } from '../shared/compile.test';

const paths = getPaths('.');

async function getFile(path: string) {
  // @ts-ignore
  const basePath = new URL('./../../', import.meta.url).toString().replace(/\/$/g, '');
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

describe('noir-compiler/browser', () => {
  shouldCompileProgramIdentically(
    'simple',
    async () => {
      const { simpleScriptExpectedArtifact } = paths;
      const fm = createFileManager('/');
      const files = Object.values(paths).filter((fileOrDir) => /^\.?\/.*\..*$/.test(fileOrDir));
      for (const path of files) {
        console.log(path);
        await fm.writeFile(path, (await getFile(path)).body as ReadableStream<Uint8Array>);
      }
      const nargoArtifact = (await getPrecompiledSource(simpleScriptExpectedArtifact)) as ProgramArtifact;
      const noirWasmArtifact = await compile_program(fm, '/fixtures/simple');

      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    60 * 20e3,
  );

  shouldCompileProgramIdentically(
    'deps',
    async () => {
      const { depsScriptExpectedArtifact } = paths;
      const fm = createFileManager('/');
      const files = Object.values(paths).filter((fileOrDir) => /^\.?\/.*\..*$/.test(fileOrDir));
      for (const path of files) {
        console.log(path);
        await fm.writeFile(path, (await getFile(path)).body as ReadableStream<Uint8Array>);
      }
      const nargoArtifact = (await getPrecompiledSource(depsScriptExpectedArtifact)) as ProgramArtifact;
      const noirWasmArtifact = await compile_program(fm, '/fixtures/with-deps');

      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    60 * 20e3,
  );

  shouldCompileContractIdentically(
    'noir-contract',
    async () => {
      const { contractExpectedArtifact } = paths;
      const fm = createFileManager('/');
      const files = Object.values(paths).filter((fileOrDir) => /^\.?\/.*\..*$/.test(fileOrDir));
      for (const path of files) {
        console.log(path);
        await fm.writeFile(path, (await getFile(path)).body as ReadableStream<Uint8Array>);
      }
      const nargoArtifact = (await getPrecompiledSource(contractExpectedArtifact)) as ContractArtifact;
      const noirWasmArtifact = await compile_contract(fm, '/fixtures/noir-contract');

      return { nargoArtifact, noirWasmArtifact };
    },
    expect,
    60 * 20e3,
  );
});
