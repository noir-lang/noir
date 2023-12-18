import { ProgramArtifact, NoirProgramCompilationArtifacts } from './noir/noir_artifact';
import { NoirWasmContractCompiler } from './noir/noir-wasm-compiler';
import { FileManager } from './noir/file-manager/file-manager';
import { LogData } from './types/utils';

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
  fileManager: FileManager,
  projectPath: string,
  wasmCompiler: unknown,
  sourceMap: unknown,
): Promise<ProgramArtifact[]> {
  const compiler = await NoirWasmContractCompiler.new(fileManager, projectPath, wasmCompiler, sourceMap, {
    log: function (msg: string, _data?: LogData) {
      console.log(msg);
    },
  });
  const artifacts = await compiler.compile();
  return artifacts.map((artifact) => {
    return generateProgramArtifact(artifact);
  });
}
