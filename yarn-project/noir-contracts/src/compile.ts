import { compile } from '@noir-lang/noir_wasm';
import { dirname, join as pathJoin } from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';
import { initialiseResolver } from '@noir-lang/noir-source-resolver';
import toml from 'toml';
import { CompiledCircuit, Dependency } from './compiled_circuit.js';

const circuitsPath = pathJoin(dirname(fileURLToPath(import.meta.url)), 'circuits');

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
