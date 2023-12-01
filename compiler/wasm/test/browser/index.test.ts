// import { expect } from '@esm-bundle/chai';
// import initNoirWasm, { compile } from '@noir-lang/noir_wasm';
// import { initializeResolver } from '@noir-lang/source-resolver';
// import {
//   depsScriptExpectedArtifact,
//   depsScriptSourcePath,
//   libASourcePath,
//   libBSourcePath,
//   simpleScriptExpectedArtifact,
//   simpleScriptSourcePath,
// } from '../shared';

// beforeEach(async () => {
//   await initNoirWasm();
// });

// async function getFileContent(path: string): Promise<string> {
//   const url = new URL(path, import.meta.url);
//   const response = await fetch(url);
//   return await response.text();
// }

// // eslint-disable-next-line @typescript-eslint/no-explicit-any
// async function getPrecompiledSource(path: string): Promise<any> {
//   const compiledData = await getFileContent(path);
//   return JSON.parse(compiledData);
// }

// describe('noir wasm', () => {
//   describe('can compile script without dependencies', () => {
//     beforeEach(async () => {
//       const source = await getFileContent(simpleScriptSourcePath);
//       initializeResolver((id: string) => {
//         console.log(`Resolving source ${id}`);

//         if (typeof source === 'undefined') {
//           throw Error(`Could not resolve source for '${id}'`);
//         } else if (id !== '/main.nr') {
//           throw Error(`Unexpected id: '${id}'`);
//         } else {
//           return source;
//         }
//       });
//     });

//     it('matching nargos compilation', async () => {
//       const wasmCircuit = await compile('/main.nr');
//       const cliCircuit = await getPrecompiledSource(simpleScriptExpectedArtifact);

//       if (!('program' in wasmCircuit)) {
//         throw Error('Expected program to be present');
//       }

//       // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
//       expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
//       expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
//       expect(wasmCircuit.program.backend).to.eq(cliCircuit.backend);
//     }).timeout(20e3); // 20 seconds
//   });

//   describe('can compile script with dependencies', () => {
//     beforeEach(async () => {
//       const [scriptSource, libASource, libBSource] = await Promise.all([
//         getFileContent(depsScriptSourcePath),
//         getFileContent(libASourcePath),
//         getFileContent(libBSourcePath),
//       ]);

//       initializeResolver((file: string) => {
//         switch (file) {
//           case '/script/main.nr':
//             return scriptSource;

//           case '/lib_a/lib.nr':
//             return libASource;

//           case '/lib_b/lib.nr':
//             return libBSource;

//           default:
//             return '';
//         }
//       });
//     });

//     it('matching nargos compilation', async () => {
//       const wasmCircuit = await compile('/script/main.nr', false, {
//         root_dependencies: ['lib_a'],
//         library_dependencies: {
//           lib_a: ['lib_b'],
//         },
//       });

//       if (!('program' in wasmCircuit)) {
//         throw Error('Expected program to be present');
//       }

//       const cliCircuit = await getPrecompiledSource(depsScriptExpectedArtifact);

//       // We don't expect the hashes to match due to how `noir_wasm` handles dependencies
//       expect(wasmCircuit.program.bytecode).to.eq(cliCircuit.bytecode);
//       expect(wasmCircuit.program.abi).to.deep.eq(cliCircuit.abi);
//       expect(wasmCircuit.program.backend).to.eq(cliCircuit.backend);
//     }).timeout(20e3); // 20 seconds
//   });
// });
