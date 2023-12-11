import { compileUsingNoirWasm } from './noir_wasm';
import { initializeResolver } from './source-resolver';

async function compile(projectPath: string) {
  const esmModule = await import('../build/esm');
  console.log('hey esm');
  return compileUsingNoirWasm(projectPath, esmModule.compile, initializeResolver);
}

export { compile };
