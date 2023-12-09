import { ProgramArtifact, NoirProgramCompilationArtifacts } from './noir/noir_artifact';
import { resolve } from 'path';
import { NoirWasmContractCompiler } from './noir/noir-wasm-compiler';

/**
 * Given a Nargo output generates an Aztec-compatible contract artifact.
 * @param compiled - Noir build output.
 * @returns Aztec contract build artifact.
 */
function generateProgramArtifact({ program }: NoirProgramCompilationArtifacts, noir_version?: string): ProgramArtifact {
  return {
    noir_version,
    hash: program.hash,
    backend: program.backend,
    abi: program.abi,
  };
}
async function compileUsingNoirWasm(projectPath: string, wasmCompiler: Function): Promise<ProgramArtifact[]> {
  const compiler = NoirWasmContractCompiler.new(resolve(projectPath), wasmCompiler);
  const artifacts = await compiler.compile();
  return artifacts.map((artifact) => {
    return generateProgramArtifact(artifact);
  });
}

async function compile(projectPath: string) {
  if (typeof require !== 'undefined') {
    const cjsModule = await require('../cjs');
    return cjsModule.compile(projectPath);
    // return compileUsingNoirWasm(projectPath, cjsModule.compile);
  } else {
    const esmModule = await import('../esm');
    return esmModule.compile(projectPath);
    // return compileUsingNoirWasm(projectPath, esmModule.compile);
  }
}

export { compile };
