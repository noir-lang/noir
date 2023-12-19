import { isAbsolute } from 'path';

import { NoirDependencyManager } from './dependencies/dependency-manager';
import { GithubDependencyResolver as GithubCodeArchiveDependencyResolver } from './dependencies/github-dependency-resolver';
import { LocalDependencyResolver } from './dependencies/local-dependency-resolver';
import { FileManager } from './file-manager/file-manager';
import { NoirPackage } from './package';
import { LogData, LogFn } from '../utils';
import { NoirCompilationResult } from '../types/noir_artifact';

/** Compilation options */
export type NoirWasmCompileOptions = {
  /** Logging function */
  log: LogFn;
  /** Log debugging information through this function */
  debugLog?: LogFn;
};

/**
 * Noir Package Compiler
 */
export class NoirWasmContractCompiler {
  #log: LogFn;
  #debugLog: LogFn;
  #package: NoirPackage;
  /* eslint-disable @typescript-eslint/no-explicit-any */
  #wasmCompiler: any;
  #sourceMap: any;
  /* eslint-disable @typescript-eslint/no-explicit-any */
  #fm: FileManager;
  #dependencyManager: NoirDependencyManager;

  private constructor(
    entrypoint: NoirPackage,
    dependencyManager: NoirDependencyManager,
    fileManager: FileManager,
    wasmCompiler: unknown,
    sourceMap: unknown,
    opts: NoirWasmCompileOptions,
  ) {
    this.#log = opts.log;
    this.#debugLog =
      opts.debugLog ??
      function (msg: string, _data?: LogData) {
        console.log(msg);
      };
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
    if (!isAbsolute(projectPath)) {
      throw new Error('projectPath must be an absolute path');
    }

    const noirPackage = await NoirPackage.open(projectPath, fileManager);

    const dependencyManager = new NoirDependencyManager(
      [
        new LocalDependencyResolver(fileManager),
        new GithubCodeArchiveDependencyResolver(fileManager),
        // TODO support actual Git repositories
      ],
      noirPackage,
    );

    return new NoirWasmContractCompiler(
      noirPackage,
      dependencyManager,
      fileManager,
      wasmCompiler.compile,
      sourceMap,
      opts,
    );
  }

  /**
   * Compile EntryPoint
   */
  /**
   * Compile EntryPoint
   */
  public async compile(): Promise<NoirCompilationResult[]> {
    console.log(`Compiling at ${this.#package.getEntryPointPath()}`);

    if (!(this.#package.getType() === 'contract' || this.#package.getType() === 'bin')) {
      this.#log(
        `Compile skipped - only supports compiling "contract" and "bin" package types (${this.#package.getType()})`,
      );
      return [];
    }
    await this.#dependencyManager.resolveDependencies();
    this.#debugLog(`Dependencies: ${this.#dependencyManager.getPackageNames().join(', ')}`);

    try {
      const isContract: boolean = this.#package.getType() === 'contract';

      const entrypoint = this.#package.getEntryPointPath();
      const deps = {
        /* eslint-disable camelcase */
        root_dependencies: this.#dependencyManager.getEntrypointDependencies(),
        library_dependencies: this.#dependencyManager.getLibraryDependencies(),
        /* eslint-enable camelcase */
      };
      const packageSources = await this.#package.getSources(this.#fm);
      const librarySources = (
        await Promise.all(
          this.#dependencyManager
            .getLibraries()
            .map(async ([alias, library]) => await library.package.getSources(this.#fm, alias)),
        )
      ).flat();
      this.#sourceMap.clean();
      [...packageSources, ...librarySources].forEach((sourceFile) => {
        this.#sourceMap.add_source_code(sourceFile.path, sourceFile.source);
      });
      const result = this.#wasmCompiler.compile(entrypoint, isContract, deps, this.#sourceMap);

      if ((isContract && !('contract' in result)) || (!isContract && !('program' in result))) {
        throw new Error('Invalid compilation result');
      }

      return [result];
    } catch (err) {
      if (err instanceof Error && err.name === 'CompileError') {
        await this.#processCompileError(err);
        throw new Error('Compilation failed');
      }

      throw err;
    }
  }

  async #resolveFile(path: string) {
    try {
      const libFile = this.#dependencyManager.findFile(path);
      return await this.#fm.readFile(libFile ?? path, 'utf-8');
    } catch (err) {
      return '';
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  async #processCompileError(err: any): Promise<void> {
    for (const diag of err.diagnostics) {
      this.#log(`  ${diag.message}`);
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
        this.#log(`    ${diag.file}:${errorLine}: ${contents.slice(secondary.start, secondary.end)}`);
      }
    }
  }
}
