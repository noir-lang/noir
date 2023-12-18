import { compileUsingNoirWasm } from './noir_wasm';
import { FileManager } from './noir/file-manager/file-manager';

async function compile(fileManager: FileManager, projectPath: string) {
  const cjsModule = await require('../build/cjs');
  console.log('hey cjs');
  return compileUsingNoirWasm(fileManager, projectPath, cjsModule.compile, new cjsModule.PathToFileSourceMap());
}

export { compile };
