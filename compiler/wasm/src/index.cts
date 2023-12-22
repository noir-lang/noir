import { FileManager } from './noir/file-manager/file-manager';
import { createNodejsFileManager } from './noir/file-manager/nodejs-file-manager';
import { NoirWasmCompiler } from './noir/noir-wasm-compiler';
import { LogData, LogFn } from './utils';
import { CompiledCircuit } from './types/noir_artifact';

async function compile(fileManager: FileManager, projectPath?: string, logFn?: LogFn) {
  const cjs = await require('../build/cjs');
  const compiler = await NoirWasmCompiler.new(
    fileManager,
    projectPath ?? fileManager.getDataDir(),
    cjs,
    new cjs.PathToFileSourceMap(),
    {
      log:
        logFn ??
        function (msg: string, _data?: LogData) {
          console.log(msg);
        },
    },
  );
  return await compiler.compile();
}

const createFileManager = createNodejsFileManager;

export { compile, createFileManager, CompiledCircuit };
