import { ContractAbi, FunctionType } from '@aztec/foundation/abi';

import noirResolver from '@noir-lang/noir-source-resolver';
import { compile } from '@noir-lang/noir_wasm';
import fsSync from 'fs';
import fs from 'fs/promises';
import nodePath from 'path';
import toml from 'toml';

import { mockVerificationKey } from './mockedKeys.js';
import { NoirCompiledContract } from './noir_artifact.js';

/**
 * A dependency entry of Nargo.toml.
 */
export interface Dependency {
  /**
   * Path to the dependency.
   */
  path?: string;
  /**
   * Git repository of the dependency.
   */
  git?: string;
}

/**
 * A class that compiles noir contracts and outputs the Aztec ABI.
 */
export class ContractCompiler {
  constructor(private projectPath: string) {}

  /**
   * Compiles the contracts in projectPath and returns the Aztec ABI.
   * @returns Aztec ABI of the compiled contracts.
   */
  public async compile(): Promise<ContractAbi[]> {
    const noirContracts = await this.compileNoir();
    const abis: ContractAbi[] = noirContracts.map(this.convertToAztecABI);
    return abis;
  }

  /**
   * Converts a compiled noir contract to Aztec ABI.
   * @param contract - A compiled noir contract.
   * @returns Aztec ABI of the contract.
   */
  private convertToAztecABI(contract: NoirCompiledContract): ContractAbi {
    return {
      ...contract,
      functions: contract.functions.map(noirFn => ({
        name: noirFn.name,
        functionType: noirFn.function_type.toLowerCase() as FunctionType,
        isInternal: noirFn.is_internal,
        parameters: noirFn.abi.parameters,
        returnTypes: [noirFn.abi.return_type],
        bytecode: Buffer.from(noirFn.bytecode).toString('hex'),
        verificationKey: mockVerificationKey,
      })),
    };
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

    return compile({
      contracts: true,
      optional_dependencies_set: Object.keys(dependenciesMap), // eslint-disable-line camelcase
    });
  }
}
