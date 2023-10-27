import { ContractArtifact } from '@aztec/foundation/abi';

import * as fs from 'node:fs';
import { join, resolve } from 'path';

import { CompileOpts, NargoContractCompiler } from './compile/nargo.js';
import { FileManager } from './compile/noir/file-manager/file-manager.js';
import { NoirWasmCompileOptions, NoirWasmContractCompiler } from './compile/noir/noir-wasm-compiler.js';
import { generateContractArtifact } from './contract-interface-gen/abi.js';

export * from './noir-version.js';

export { generateNoirContractInterface } from './contract-interface-gen/noir.js';
export { generateTypescriptContractInterface } from './contract-interface-gen/typescript.js';
export { generateContractArtifact };

export * from './noir_artifact.js';

/**
 * Compile Aztec.nr contracts in project path using a nargo binary available in the shell.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNargo(projectPath: string, opts: CompileOpts = {}): Promise<ContractArtifact[]> {
  return (await new NargoContractCompiler(projectPath, opts).compile()).map(generateContractArtifact);
}

/**
 * Compile Aztec.nr contracts in project path using built-in noir_wasm.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNoirWasm(
  projectPath: string,
  opts: NoirWasmCompileOptions,
): Promise<ContractArtifact[]> {
  const cacheRoot = process.env.XDG_CACHE_HOME ?? join(process.env.HOME ?? '', '.cache');
  const fileManager = new FileManager(fs, join(cacheRoot, 'aztec-noir-compiler'));

  return (await NoirWasmContractCompiler.new(fileManager, resolve(projectPath), opts).compile()).map(
    generateContractArtifact,
  );
}
