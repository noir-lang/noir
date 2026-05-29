import { compile, createFileManager } from 'noir-wasm-published';

export async function compileCircuitFromSource({ nargoToml, mainNr }) {
  const fileManager = createFileManager('/');
  await fileManager.writeFile('./Nargo.toml', nargoToml);
  await fileManager.writeFile('./src/main.nr', mainNr);
  const result = await compile(fileManager);
  if (!('program' in result)) {
    throw new Error('Noir circuit compilation failed.');
  }
  return result.program;
}

export async function compileCircuitFromFiles(baseUrl = new URL('../', import.meta.url)) {
  const [{ readFile }, { resolve }] = await Promise.all([import('node:fs/promises'), import('node:path')]);
  const root = baseUrl instanceof URL ? baseUrl : new URL(baseUrl);
  const nargoToml = await readFile(resolve(root.pathname, 'Nargo.toml'), 'utf8');
  const mainNr = await readFile(resolve(root.pathname, 'src/main.nr'), 'utf8');
  return compileCircuitFromSource({ nargoToml, mainNr });
}
