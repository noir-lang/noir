export async function compileCircuitFromSource({ nargoToml, mainNr }) {
  const compiler = await loadNoirWasmCompiler();
  const { compile, createFileManager } = compiler;
  if (compiler.webFileManager) {
    const fileManager = createFileManager('/');
    await fileManager.writeFile('./Nargo.toml', stringToStream(nargoToml));
    await fileManager.writeFile('./src/main.nr', stringToStream(mainNr));
    const result = await compile(fileManager);
    if (!('program' in result)) {
      throw new Error('Noir circuit compilation failed.');
    }
    return result.program;
  }

  const [{ mkdtemp, mkdir, rm }, { tmpdir }, { join }] = await Promise.all([
    import('node:fs/promises'),
    import('node:os'),
    import('node:path'),
  ]);
  const projectPath = await mkdtemp(join(tmpdir(), 'usb-auth-noir-'));
  await mkdir(join(projectPath, 'src'), { recursive: true });
  const fileManager = createFileManager(projectPath);
  await fileManager.writeFile('Nargo.toml', stringToStream(nargoToml));
  await fileManager.writeFile(join('src', 'main.nr'), stringToStream(mainNr));
  const result = await compile(fileManager);
  await rm(projectPath, { recursive: true, force: true });
  if (!('program' in result)) {
    throw new Error('Noir circuit compilation failed.');
  }
  return result.program;
}

function stringToStream(value) {
  return new Response(value).body;
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
    const fallback = resolve(repoRoot, 'demo/client/node_modules/@noir-lang/noir_wasm/dist/web/main.mjs');
    try {
      await access(fallback);
      globalThis.self ??= globalThis;
      globalThis.document ??= { baseURI: pathToFileURL(fallback).href };
      globalThis.location ??= { href: pathToFileURL(fallback).href };
      const compiler = await import(pathToFileURL(fallback).href);
      return { ...compiler, webFileManager: true };
    } catch (_fallbackError) {
      throw new Error('No built Noir WASM compiler found. Build @noir-lang/noir_wasm or run nargo compile first.');
    }
  }
}
