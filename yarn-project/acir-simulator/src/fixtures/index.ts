import { compile } from '@noir-lang/noir_wasm';
import { dirname, join as pathJoin } from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';
import { initialiseResolver } from '@noir-lang/noir-source-resolver';
import toml from 'toml';

const circuitsPath = pathJoin(dirname(fileURLToPath(import.meta.url)), 'circuits');

/**
 * A dependency entry of Nargo.toml.
 */
interface Dependency {
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
 * A circuit type.
 */
interface CircuitType {
  /**
   * The actual type.
   */
  kind: any;
}

/**
 * A parameter to the circuit.
 */
interface Parameter {
  /**
   * The name of the parameter.
   */
  name: string;
  /**
   * The type of the parameter.
   */
  type: CircuitType;
  /**
   * The visibility of the parameter.
   */
  visibility: 'private' | 'public';
}

/**
 * The representation of a compiled circuit.
 */
interface CompiledCircuit {
  /**
   * The bytecode of the circuit.
   */
  circuit: Array<number>;
  /**
   * The Noir ABI of the circuit.
   */
  abi: {
    /**
     * The circuit  parameters.
     */
    parameters: Array<Parameter>;
    /**
     * The witness indices for the parameters.
     */
    param_witnesses: Record<string, Array<number>>;
    /**
     * The circuit return type.
     */
    return_type: CircuitType | null;
    /**
     * The witness indices for the return value.
     */
    return_witnesses: Array<number>;
  };
}

/**
 *
 * @param circuitPath - Path to the circuit crate.
 * @returns The dependencies of the circuit as a map of name to dependency entry.
 */
function readDependencies(circuitPath: string) {
  const { dependencies } = toml.parse(fs.readFileSync(pathJoin(circuitPath, 'Nargo.toml'), { encoding: 'utf8' }));
  return dependencies as Record<string, Dependency>;
}

/**
 * Compiles a noir circuit fixture by name.
 * @param circuitName - Name of the circuit fixture to compile.
 * @returns The compiled circuit.
 */
export function compileCircuit(circuitName: string) {
  const circuitPath = pathJoin(circuitsPath, circuitName);
  const dependenciesMap = readDependencies(circuitPath);

  initialiseResolver((id: `${string}/lib.nr` | 'main.nr') => {
    let path;
    if (id === 'main.nr') {
      path = pathJoin(circuitPath, 'src/main.nr');
    } else {
      const [dependencyName] = id.split('/');
      const dependency = dependenciesMap[dependencyName];
      if (!dependency.path) {
        throw new Error(`Don't know how to resolve dependency ${dependencyName}`);
      }
      path = pathJoin(circuitPath, dependency.path, 'src/lib.nr');
    }
    const result = fs.readFileSync(path, { encoding: 'utf8' });
    return result;
  });

  return compile({
    optional_dependencies_set: Object.keys(dependenciesMap), // eslint-disable-line camelcase
  }) as CompiledCircuit;
}
