import { FileManager } from './noir/file-manager/file-manager';
import { compileUsingNoirWasm } from './noir_wasm';

async function compile(fileManager: FileManager, projectPath: string) {
  const esmModule = await import('../build/esm');
  console.log('hey esm');
  return compileUsingNoirWasm(fileManager, projectPath, esmModule.compile, new esmModule.PathToFileSourceMap());
}

export { compile };
