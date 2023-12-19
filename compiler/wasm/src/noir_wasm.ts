import { NoirCompilationResult } from './types/noir_artifact';
import { NoirWasmContractCompiler } from './noir/noir-wasm-compiler';
import { FileManager } from './noir/file-manager/file-manager';
import { LogData } from './utils';

export async function compileUsingNoirWasm(
  fileManager: FileManager,
  projectPath: string,
  wasmCompiler: unknown,
  sourceMap: unknown,
): Promise<NoirCompilationResult[]> {
  const compiler = await NoirWasmContractCompiler.new(fileManager, projectPath, wasmCompiler, sourceMap, {
    log: function (msg: string, _data?: LogData) {
      console.log(msg);
    },
  });
  return await compiler.compile();
}
