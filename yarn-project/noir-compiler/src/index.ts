import { ContractAbi } from '@aztec/foundation/abi';

import { CompileOpts, NargoContractCompiler } from './compile/nargo.js';
import { generateAztecAbi } from './contract-interface-gen/abi.js';
import NoirVersion from './noir-version.json' assert { type: 'json' };

const { commit: NoirCommit, tag: NoirTag } = NoirVersion;
export { NoirCommit, NoirTag };

export { generateNoirContractInterface } from './contract-interface-gen/noir.js';
export { generateTypescriptContractInterface } from './contract-interface-gen/typescript.js';
export { generateAztecAbi };

/**
 * Compile Aztec.nr contracts in project path using a nargo binary available in the shell.
 * @param projectPath - Path to project.
 * @param opts - Compiler options.
 * @returns Compiled artifacts.
 */
export async function compileUsingNargo(projectPath: string, opts: CompileOpts = {}): Promise<ContractAbi[]> {
  return (await new NargoContractCompiler(projectPath, opts).compile()).map(generateAztecAbi);
}
