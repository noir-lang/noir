import { FileManager } from './noir/file-manager/file-manager';
import { createNodejsFileManager } from './noir/file-manager/nodejs-file-manager';
import { NoirWasmCompiler } from './noir/noir-wasm-compiler';
import { LogData, LogFn } from './utils';
import { CompilationResult } from './types/noir_artifact';
import { inflateDebugSymbols } from './noir/debug';

async function compile(
  fileManager: FileManager,
  projectPath?: string,
  logFn?: LogFn,
  debugLogFn?: LogFn,
): Promise<CompilationResult> {
  if (logFn && !debugLogFn) {
    debugLogFn = logFn;
  }

  const esm = await import(/* webpackMode: "eager" */ '../build/esm');
  await esm.default();

  const compiler = await NoirWasmCompiler.new(
    fileManager,
    projectPath ?? fileManager.getDataDir(),
    esm,
    new esm.PathToFileSourceMap(),
    {
      log:
        logFn ??
        function (msg: string, data?: LogData) {
          if (data) {
            console.log(msg, data);
          } else {
            console.log(msg);
          }
        },
      debugLog:
        debugLogFn ??
        function (msg: string, data?: LogData) {
          if (data) {
            console.debug(msg, data);
          } else {
            console.debug(msg);
          }
        },
    },
  );
  return await compiler.compile();
}

const createFileManager = createNodejsFileManager;

export { compile, createFileManager, inflateDebugSymbols, CompilationResult };
