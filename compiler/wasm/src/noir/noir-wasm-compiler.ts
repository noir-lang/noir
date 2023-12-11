import { isAbsolute } from 'path';

import { NoirProgramCompilationArtifacts } from './noir_artifact';
import { NoirDependencyManager } from './dependencies/dependency-manager';
import { GithubDependencyResolver as GithubCodeArchiveDependencyResolver } from './dependencies/github-dependency-resolver';
import { LocalDependencyResolver } from './dependencies/local-dependency-resolver';
import { NoirPackage } from './package';
import { readFileSync } from 'fs';

/**
 * Noir Package Compiler
 */
export class NoirWasmContractCompiler {
  #package: NoirPackage;
  #dependencyManager: NoirDependencyManager;
  #wasmCompiler: any;
  resolver: Function;

  private constructor(
    entrypoint: NoirPackage,
    dependencyManager: NoirDependencyManager,
    wasmCompiler: any,
    resolver: Function,
  ) {
    this.#package = entrypoint;
    this.#dependencyManager = dependencyManager;
    this.#wasmCompiler = wasmCompiler;
    this.resolver = resolver;
  }

  /**
   * Creates a new compiler instance.
   * @param fileManager - The file manager to use
   * @param projectPath - The path to the project
   * @param opts - Compilation options
   */
  public static new(projectPath: string, wasmCompiler: any, resolver: Function) {
    const noirPackage = NoirPackage.open(projectPath);

    const dependencyManager = new NoirDependencyManager(
      [
        new LocalDependencyResolver(),
        new GithubCodeArchiveDependencyResolver(),
        // TODO support actual Git repositories
      ],
      noirPackage,
    );

    return new NoirWasmContractCompiler(noirPackage, dependencyManager, wasmCompiler, resolver);
  }

  /**
   * Gets the version of Aztec.nr that was used compiling this contract.
   */
  public getResolvedAztecNrVersion() {
    // TODO eliminate this hardcoded library name!
    // see docs/docs/dev_docs/contracts/setup.md
    return this.#dependencyManager.getVersionOf('aztec');
  }

  /**
   * Compile EntryPoint
   */
  public async compile(): Promise<NoirProgramCompilationArtifacts[]> {
    console.log(`Compiling Program at ${this.#package.getEntryPointPath()}`);
    return await this.compileProgram();
  }

  /**
   * Compiles the Program.
   */
  public async compileProgram(): Promise<NoirProgramCompilationArtifacts[]> {
    await this.#dependencyManager.resolveDependencies();
    console.log(`Dependencies: ${this.#dependencyManager.getPackageNames().join(', ')}`);

    this.resolver(this.#resolveFile);

    try {
      const isContract: boolean = false;
      const result = this.#wasmCompiler(this.#package.getEntryPointPath(), isContract, {
        /* eslint-disable camelcase */
        root_dependencies: this.#dependencyManager.getEntrypointDependencies(),
        library_dependencies: this.#dependencyManager.getLibraryDependencies(),
        /* eslint-enable camelcase */
      });

      if (!('program' in result)) {
        throw new Error('No program found in compilation result');
      }

      return [{ name: this.#package.getNoirPackageConfig().package.name, ...result }];
    } catch (err) {
      if (err instanceof Error && err.name === 'CompileError') {
        this.#processCompileError(err as any);
      }

      throw err;
    }
  }

  #resolveFile = (path: string) => {
    try {
      const libFile = this.#dependencyManager.findFile(path);

      const data = readFileSync(libFile ?? path, 'utf-8') as string;
      return data;
    } catch (err) {
      return '';
    }
  };

  #processCompileError(err: any): void {
    for (const diag of err.diagnostics) {
      console.log(`  ${diag.message}`);
      const contents = this.#resolveFile(diag.file);
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
        console.log(`    ${diag.file}:${errorLine}: ${contents.slice(secondary.start, secondary.end)}`);
      }
    }
  }
}
