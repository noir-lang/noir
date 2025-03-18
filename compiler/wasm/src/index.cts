import { FileManager } from './noir/file-manager/file-manager';
import { createNodejsFileManager } from './noir/file-manager/nodejs-file-manager';
import { NoirWasmCompiler } from './noir/noir-wasm-compiler';
import { LogData, LogFn } from './utils';
import { ContractCompilationArtifacts, ProgramCompilationArtifacts } from './types/noir_artifact';
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
 * import { compile_program, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager(myProjectPath);
 * const myCompiledCode = await compile_program(fm);
 * ```
 *
 * ```typescript
 * // Browser
 *
 * import { compile_program, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager('/');
 * for (const path of files) {
 *   await fm.writeFile(path, await getFileAsStream(path));
 * }
 * const myCompiledCode = await compile_program(fm);
 * ```
 */
async function compile_program(
  fileManager: FileManager,
  projectPath?: string,
  logFn?: LogFn,
  debugLogFn?: LogFn,
): Promise<ProgramCompilationArtifacts> {
  const compiler = await setup_compiler(fileManager, projectPath, logFn, debugLogFn);
  return await compiler.compile_program();
}

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
 * import { compile_contract, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager(myProjectPath);
 * const myCompiledCode = await compile_contract(fm);
 * ```
 *
 * ```typescript
 * // Browser
 *
 * import { compile_contract, createFileManager } from '@noir-lang/noir_wasm';
 *
 * const fm = createFileManager('/');
 * for (const path of files) {
 *   await fm.writeFile(path, await getFileAsStream(path));
 * }
 * const myCompiledCode = await compile_contract(fm);
 * ```
 */
async function compile_contract(
  fileManager: FileManager,
  projectPath?: string,
  logFn?: LogFn,
  debugLogFn?: LogFn,
): Promise<ContractCompilationArtifacts> {
  const compiler = await setup_compiler(fileManager, projectPath, logFn, debugLogFn);
  return await compiler.compile_contract();
}

async function setup_compiler(
  fileManager: FileManager,
  projectPath?: string,
  logFn?: LogFn,
  debugLogFn?: LogFn,
): Promise<NoirWasmCompiler> {
  if (logFn && !debugLogFn) {
    debugLogFn = logFn;
  }

  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const cjs = await require('../build/cjs');
  return await NoirWasmCompiler.new(
    fileManager,
    projectPath ?? fileManager.getDataDir(),
    cjs,
    new cjs.PathToFileSourceMap(),
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
}

const createFileManager = createNodejsFileManager;

export {
  compile_program as compile,
  compile_program,
  compile_contract,
  createFileManager,
  inflateDebugSymbols,
  ProgramCompilationArtifacts,
  ContractCompilationArtifacts,
};
