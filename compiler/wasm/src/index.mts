import { FileManager } from './noir/file-manager/file-manager';
import { createNodejsFileManager } from './noir/file-manager/nodejs-file-manager';
import { NoirWasmCompiler } from './noir/noir-wasm-compiler';
import { LogData, LogFn } from './utils';
import { CompilationResult } from './types/noir_artifact';
import { inflateDebugSymbols } from './noir/debug';

/**
 * Compiles a Noir project
 *
 * @param fileManager - The file manager to use
 * @param projectPath - The path to the project inside the file manager. Defaults to the root of the file manager
 * @param logFn - A logging function. If not provided, console.log will be used
 * @param debugLogFn - A debug logging function. If not provided, logFn will be used
 *
 * @example
 * ```typescript
 * // Node.js
 *
 * import { compile, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager(myProjectPath);
 * const myCompiledCode = await compile(fm);
 * ```
 *
 * ```typescript
 * // Browser
 *
 * import { compile, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager('/');
 * for (const path of files) {
 *   await fm.writeFile(path, await getFileAsStream(path));
 * }
 * const myCompiledCode = await compile(fm);
 * ```
 */
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
