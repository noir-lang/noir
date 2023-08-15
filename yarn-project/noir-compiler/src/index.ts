import { ContractAbi } from '@aztec/foundation/abi';

import { CompileOpts, NargoContractCompiler } from './compile/nargo.js';
import { WasmContractCompiler } from './compile/wasm.js';
import { generateAztecAbi } from './contract-interface-gen/abi.js';

export { generateAztecAbi };
export { generateNoirContractInterface } from './contract-interface-gen/noir.js';
export { generateTypescriptContractInterface } from './contract-interface-gen/typescript.js';

/**
 * Compile Noir contracts in project path using the noir-lang/noir-wasm package.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNoirWasm(projectPath: string): Promise<ContractAbi[]> {
  return (await new WasmContractCompiler(projectPath).compile()).map(generateAztecAbi);
}

/**
 * Compile Noir contracts in project path using a nargo binary available in the shell.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNargo(projectPath: string, opts: CompileOpts = {}): Promise<ContractAbi[]> {
  return (await new NargoContractCompiler(projectPath, opts).compile()).map(generateAztecAbi);
}
