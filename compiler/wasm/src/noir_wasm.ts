import { ProgramArtifact, NoirProgramCompilationArtifacts } from './noir/noir_artifact';
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
export async function compileUsingNoirWasm(
  projectPath: string,
  wasmCompiler: Function,
  resolver: Function,
): Promise<ProgramArtifact[]> {
  const compiler = NoirWasmContractCompiler.new(projectPath, wasmCompiler, resolver);
  const artifacts = await compiler.compile();
  return artifacts.map((artifact) => {
    return generateProgramArtifact(artifact);
  });
}
