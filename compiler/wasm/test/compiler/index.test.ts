// import { ContractArtifact } from '@aztec/foundation/abi';
// import { LogFn, createDebugLogger } from '@aztec/foundation/log';
// import { fileURLToPath } from '@aztec/foundation/url';

// import { execSync } from 'child_process';
// import path from 'path';

// import {
//   ProgramArtifact,
//   compileUsingNargo,
//   compileUsingNoirWasm,
//   generateNoirContractInterface,
//   generateTypescriptContractInterface,
// } from './index.js';

// function isNargoAvailable() {
//   try {
//     execSync(`which nargo`);
//     return true;
//   } catch (error) {
//     return false;
//   }
// }

// describe('noir-compiler', () => {
//   let projectPath: string;
//   let log: LogFn;
//   beforeAll(() => {
//     const currentDirName = path.dirname(fileURLToPath(import.meta.url));
//     projectPath = path.join(currentDirName, 'fixtures/test_contract');
//     log = createDebugLogger('noir-compiler:test');
//   });

//   const nargoAvailable = isNargoAvailable();
//   const conditionalDescribe = nargoAvailable ? describe : describe.skip;
//   const conditionalIt = nargoAvailable ? it : it.skip;
//   const withoutDebug = ({
//     debug: _debug,
//     ...rest
//   }: ContractArtifact | ProgramArtifact): Omit<ContractArtifact | ProgramArtifact, 'debug'> => rest;

//   function compilerTest(
//     compileFn: (path: string, opts: { log: LogFn }) => Promise<(ProgramArtifact | ContractArtifact)[]>,
//   ) {
//     let compiled: (ProgramArtifact | ContractArtifact)[];
//     let compiledContract: ContractArtifact[];

//     beforeAll(async () => {
//       compiled = await compileFn(projectPath, { log });
//       compiledContract = compiled.map(_compiled => _compiled as ContractArtifact);
//     });

//     it('compiles the test contract', () => {
//       expect(compiledContract.map(withoutDebug)).toMatchSnapshot();
//     });

//     it('generates typescript interface', () => {
//       const result = generateTypescriptContractInterface(compiledContract[0], `../target/test.json`);
//       expect(result).toMatchSnapshot();
//     });

//     it('generates Aztec.nr external interface', () => {
//       const result = generateNoirContractInterface(compiledContract[0]);
//       expect(result).toMatchSnapshot();
//     });
//   }

//   describe('using wasm binary', () => {
//     compilerTest(compileUsingNoirWasm);
//   });

//   conditionalDescribe('using nargo', () => {
//     compilerTest(compileUsingNargo);
//   });

//   conditionalIt('both nargo and noir_wasm should compile identically', async () => {
//     const [noirWasmArtifact, nargoArtifact] = await Promise.all([
//       compileUsingNoirWasm(projectPath, { log }),
//       compileUsingNargo(projectPath, { log }),
//     ]);

//     expect(nargoArtifact.map(withoutDebug)).toEqual(noirWasmArtifact.map(withoutDebug));
//   });
// });
