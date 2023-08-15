/* eslint-disable camelcase */
import noirResolver from '@noir-lang/noir-source-resolver';
import { compile } from '@noir-lang/noir_wasm';
import { fromByteArray } from 'base64-js';
import fsSync from 'fs';
import fs from 'fs/promises';
import nodePath from 'path';
import toml from 'toml';

import { NoirCompiledContract } from '../noir_artifact.js';

/** A dependency entry of Nargo.toml. */
interface Dependency {
  /** Path to the dependency. */
  path?: string;
  /** Git repository of the dependency. */
  git?: string;
}

/**
 * A class that compiles noir contracts using the noir wasm package.
 */
export class WasmContractCompiler {
  constructor(private projectPath: string) {}

  /**
   * Compiles the contracts in projectPath and returns the Noir artifact.
   * @returns Noir artifact of the compiled contracts.
   */
  public compile(): Promise<NoirCompiledContract[]> {
    return this.compileNoir();
  }

  /**
   * Reads the dependencies of a noir crate.
   * @param cratePath - Path to the noir crate.
   * @returns A map of dependencies.
   */
  private async readDependencies(cratePath: string) {
    const { dependencies } = toml.parse(
      await fs.readFile(nodePath.join(cratePath, 'Nargo.toml'), { encoding: 'utf8' }),
    );
    return (dependencies || {}) as Record<string, Dependency>;
  }

  /**
   * Cleans up wasm output and formats it to match nargo output.
   * @param contract - A contract as outputted by wasm.
   * @returns A nargo-like contract artifact.
   */
  private cleanUpWasmOutput(contract: any): NoirCompiledContract {
    return {
      ...contract,
      functions: contract.functions.map((fn: any) => ({
        ...fn,
        is_internal: !!fn.is_internal, // noir wasm may return undefined for is_internal
        bytecode: fromByteArray(fn.bytecode), // wasm returns Uint8Array instead of base64-encoded bytecode
      })),
    };
  }

  /**
   * Executes the noir compiler.
   * @returns A list of compiled noir contracts.
   */
  private async compileNoir(): Promise<NoirCompiledContract[]> {
    const dependenciesMap = await this.readDependencies(this.projectPath);

    /**
     * The resolver receives a relative path, and the first part of the path can be a dependency name.
     * If the dependency is found in the map, the rest of the path inside that dependency src folder.
     * Otherwise, resolve the full relative path requested inside the project path.
     */
    noirResolver.initialiseResolver((id: string) => {
      const idParts = id.split('/');

      let path;
      if (dependenciesMap[idParts[0]]) {
        const [dependencyName, ...dependencySubpathParts] = idParts;
        const dependency = dependenciesMap[dependencyName];
        if (!dependency.path) {
          throw new Error(`Don't know how to resolve dependency ${dependencyName}`);
        }
        path = nodePath.resolve(this.projectPath, dependency.path, 'src', dependencySubpathParts.join('/'));
      } else {
        path = nodePath.join(this.projectPath, 'src', idParts.join('/'));
      }

      // The resolver does not support async resolution
      // and holding the whole project in memory is not reasonable
      const result = fsSync.readFileSync(path, { encoding: 'utf8' });
      return result;
    });

    const result = await compile({
      contracts: true,
      optional_dependencies_set: Object.keys(dependenciesMap), // eslint-disable-line camelcase
    });

    return result.map(this.cleanUpWasmOutput);
  }
}
