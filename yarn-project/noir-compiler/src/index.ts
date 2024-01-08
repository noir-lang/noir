import { ContractArtifact } from '@aztec/foundation/abi';

import { join, resolve } from 'path';

import { CompileOpts, NargoContractCompiler } from './compile/nargo.js';
import { createNodejsFileManager } from './compile/noir/file-manager/nodejs-file-manager.js';
import { NoirWasmCompileOptions, NoirWasmContractCompiler } from './compile/noir/noir-wasm-compiler.js';
import { generateArtifact, generateContractArtifact } from './contract-interface-gen/abi.js';
import { ProgramArtifact } from './noir_artifact.js';

export * from './versions.js';

export { generateTypescriptContractInterface } from './contract-interface-gen/contractTypescript.js';
export { generateNoirContractInterface } from './contract-interface-gen/noir.js';
export { generateTypescriptProgramInterface } from './contract-interface-gen/programTypescript.js';
export { generateContractArtifact };

export * from './noir_artifact.js';

/**
 * Compile Aztec.nr contracts in project path using a nargo binary available in the shell.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNargo(projectPath: string, opts: CompileOpts = {}): Promise<ContractArtifact[]> {
  return (await new NargoContractCompiler(projectPath, opts).compile()).map(artifact =>
    generateContractArtifact(artifact),
  );
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
): Promise<(ContractArtifact | ProgramArtifact)[]> {
  const cacheRoot = process.env.XDG_CACHE_HOME ?? join(process.env.HOME ?? '', '.cache');
  const fileManager = createNodejsFileManager(join(cacheRoot, 'aztec-noir-compiler'));
  const compiler = await NoirWasmContractCompiler.new(fileManager, resolve(projectPath), opts);
  const artifacts = await compiler.compile();
  return artifacts.map(artifact => {
    return generateArtifact(artifact);
  });
}
