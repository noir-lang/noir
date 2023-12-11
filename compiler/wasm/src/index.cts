import { compileUsingNoirWasm } from './noir_wasm';
import { initializeResolver } from './source-resolver';

async function compile(projectPath: string) {
  const cjsModule = await require('../build/cjs');
  console.log('hey cjs');
  return compileUsingNoirWasm(projectPath, cjsModule.compile, initializeResolver);
}

export { compile };
