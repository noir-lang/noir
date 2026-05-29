export async function compileCircuitFromSource({ nargoToml, mainNr }) {
  const { compile, createFileManager } = await loadNoirWasmCompiler();
  const [{ mkdtemp, mkdir, writeFile, rm }, { tmpdir }, { join }] = await Promise.all([
    import('node:fs/promises'),
    import('node:os'),
    import('node:path'),
  ]);
  const projectPath = await mkdtemp(join(tmpdir(), 'usb-auth-noir-'));
  await mkdir(join(projectPath, 'src'), { recursive: true });
  await writeFile(join(projectPath, 'Nargo.toml'), nargoToml, 'utf8');
  await writeFile(join(projectPath, 'src/main.nr'), mainNr, 'utf8');
  const fileManager = createFileManager(projectPath);
  const result = await compile(fileManager);
  await rm(projectPath, { recursive: true, force: true });
  if (!('program' in result)) {
    throw new Error('Noir circuit compilation failed.');
  }
  return result.program;
}


export async function compileCircuitFromFiles(baseUrl = new URL('../', import.meta.url)) {
  const [{ readFile }, { resolve }, { fileURLToPath }] = await Promise.all([
    import('node:fs/promises'),
    import('node:path'),
    import('node:url'),
  ]);
  const rootPath = baseUrl instanceof URL ? fileURLToPath(baseUrl) : String(baseUrl);
  const nargoToml = await readFile(resolve(rootPath, 'Nargo.toml'), 'utf8');
  const mainNr = await readFile(resolve(rootPath, 'src/main.nr'), 'utf8');
  return compileCircuitFromSource({ nargoToml, mainNr });
}

async function loadNoirWasmCompiler() {
  try {
    return await import('@noir-lang/noir_wasm');
  } catch (_workspaceError) {
    const [{ access }, { pathToFileURL, fileURLToPath }, { resolve }] = await Promise.all([
      import('node:fs/promises'),
      import('node:url'),
      import('node:path'),
    ]);
    const repoRoot = resolve(fileURLToPath(new URL('../../../', import.meta.url)));
    const fallback = resolve(repoRoot, 'demo/client/node_modules/@noir-lang/noir_wasm/dist/node/main.js');
    try {
      await access(fallback);
      return import(pathToFileURL(fallback).href);
    } catch (_fallbackError) {
      throw new Error('No built Noir WASM compiler found. Build @noir-lang/noir_wasm or run nargo compile first.');
    }
  }
}
