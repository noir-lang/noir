import { isAbsolute } from 'path';

import { DependencyManager } from './dependencies/dependency-manager';
import { GithubDependencyResolver as GithubCodeArchiveDependencyResolver } from './dependencies/github-dependency-resolver';
import { LocalDependencyResolver } from './dependencies/local-dependency-resolver';
import { FileManager } from './file-manager/file-manager';
import { Package } from './package';
import { LogFn } from '../utils';
import { ContractCompilationArtifacts, ProgramCompilationArtifacts } from '../types/noir_artifact';

/** Compilation options */
export type NoirWasmCompileOptions = {
  /** Logging function */
  log: LogFn;
  /** Log debugging information through this function */
  debugLog: LogFn;
};

/**
 * Noir Package Compiler
 */
export class NoirWasmCompiler {
  #log: LogFn;
  #debugLog: LogFn;
  #package: Package;
  /* eslint-disable @typescript-eslint/no-explicit-any */
  #wasmCompiler: any;
  #sourceMap: any;
  #fm: FileManager;
  #dependencyManager: DependencyManager;

  private constructor(
    entrypoint: Package,
    dependencyManager: DependencyManager,
    fileManager: FileManager,
    wasmCompiler: unknown,
    sourceMap: unknown,
    opts: NoirWasmCompileOptions,
  ) {
    this.#log = opts.log;
    this.#debugLog = opts.debugLog;
    this.#package = entrypoint;
    this.#fm = fileManager;
    this.#wasmCompiler = wasmCompiler;
    this.#sourceMap = sourceMap;
    this.#dependencyManager = dependencyManager;
  }

  /**
   * Creates a new compiler instance.
   * @param fileManager - The file manager to use
   * @param projectPath - The path to the project
   * @param opts - Compilation options
   */
  public static async new(
    fileManager: FileManager,
    projectPath: string,
    /* eslint-disable @typescript-eslint/no-explicit-any */
    wasmCompiler: any,
    sourceMap: any,
    /* eslint-enable @typescript-eslint/no-explicit-any */
    opts: NoirWasmCompileOptions,
  ) {
    // Assume the filemanager is initialized at the project root
    if (!isAbsolute(projectPath)) {
      throw new Error('projectPath must be an absolute path');
    }

    const noirPackage = await Package.open(projectPath, fileManager);

    const dependencyManager = new DependencyManager(
      [
        new LocalDependencyResolver(fileManager),
        // use node's global fetch
        new GithubCodeArchiveDependencyResolver(fileManager, fetch),
        // TODO support actual Git repositories
      ],
      noirPackage,
    );

    return new NoirWasmCompiler(noirPackage, dependencyManager, fileManager, wasmCompiler, sourceMap, opts);
  }

  /**
   * Compile EntryPoint
   */
  public async compile_program(): Promise<ProgramCompilationArtifacts> {
    console.log(`Compiling at ${this.#package.getEntryPointPath()}`);

    if (this.#package.getType() !== 'bin') {
      throw new Error(`Expected to find package type "bin" but found ${this.#package.getType()}`);
    }
    await this.#dependencyManager.resolveDependencies();
    this.#debugLog(`Dependencies: ${this.#dependencyManager.getPackageNames().join(', ')}`);

    try {
      const entrypoint = this.#package.getEntryPointPath();
      const deps = {
        root_dependencies: this.#dependencyManager.getEntrypointDependencies(),
        library_dependencies: this.#dependencyManager.getLibraryDependencies(),
      };
      const packageSources = await this.#package.getSources(this.#fm);
      const librarySources = (
        await Promise.all(
          this.#dependencyManager
            .getLibraries()
            .map(async ([alias, library]) => await library.package.getSources(this.#fm, alias)),
        )
      ).flat();
      [...packageSources, ...librarySources].forEach((sourceFile) => {
        this.#debugLog(`Adding source ${sourceFile.path}`);
        this.#sourceMap.add_source_code(sourceFile.path, sourceFile.source);
      });
      const result = this.#wasmCompiler.compile_program(entrypoint, deps, this.#sourceMap);

      return result;
    } catch (err) {
      if (err instanceof Error && err.name === 'CompileError') {
        const logs = await this.#processCompileError(err);
        for (const log of logs) {
          this.#log(log);
        }
        throw new Error(logs.join('\n'));
      }

      throw err;
    }
  }

  /**
   * Compile EntryPoint
   */
  public async compile_contract(): Promise<ContractCompilationArtifacts> {
    console.log(`Compiling at ${this.#package.getEntryPointPath()}`);

    if (this.#package.getType() !== 'contract') {
      throw new Error(`Expected to find package type "contract" but found ${this.#package.getType()}`);
    }
    await this.#dependencyManager.resolveDependencies();
    this.#debugLog(`Dependencies: ${this.#dependencyManager.getPackageNames().join(', ')}`);

    try {
      const entrypoint = this.#package.getEntryPointPath();
      const deps = {
        root_dependencies: this.#dependencyManager.getEntrypointDependencies(),
        library_dependencies: this.#dependencyManager.getLibraryDependencies(),
      };
      const packageSources = await this.#package.getSources(this.#fm);
      const librarySources = (
        await Promise.all(
          this.#dependencyManager
            .getLibraries()
            .map(async ([alias, library]) => await library.package.getSources(this.#fm, alias)),
        )
      ).flat();
      [...packageSources, ...librarySources].forEach((sourceFile) => {
        this.#debugLog(`Adding source ${sourceFile.path}`);
        this.#sourceMap.add_source_code(sourceFile.path, sourceFile.source);
      });
      const result = this.#wasmCompiler.compile_contract(entrypoint, deps, this.#sourceMap);

      return result;
    } catch (err) {
      if (err instanceof Error && err.name === 'CompileError') {
        const logs = await this.#processCompileError(err);
        for (const log of logs) {
          this.#log(log);
        }
        throw new Error(logs.join('\n'));
      }

      throw err;
    }
  }

  async #resolveFile(path: string) {
    try {
      const libFile = this.#dependencyManager.findFile(path);
      return await this.#fm.readFile(libFile ?? path, 'utf-8');
    } catch (_err) {
      return '';
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  async #processCompileError(err: any): Promise<string[]> {
    const logs = [];
    for (const diag of err.diagnostics) {
      logs.push(`  ${diag.message}`);
      const contents = await this.#resolveFile(diag.file);
      const lines = contents.split('\n');
      const lineOffsets = lines.reduce<number[]>((accum, _, idx) => {
        if (idx === 0) {
          accum.push(0);
        } else {
          accum.push(accum[idx - 1] + lines[idx - 1].length + 1);
        }
        return accum;
      }, []);

      for (const secondary of diag.secondaries) {
        const errorLine = lineOffsets.findIndex((offset) => offset > secondary.start);
        logs.push(`    ${diag.file}:${errorLine}: ${contents.slice(secondary.start, secondary.end)}`);
      }
    }
    return logs;
  }
}
