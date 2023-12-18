import { join, resolve } from 'path';
import { fileURLToPath } from '../src/types/utils';

const fixtures = resolve(fileURLToPath(import.meta.url), '../../public/fixtures');

export const simpleScriptSourcePath = join(fixtures, 'simple/noir-script/src/main.nr');
export const simpleScriptExpectedArtifact = join(fixtures, 'simple/noir-script/target/noir_wasm_testing.json');

export const depsScriptSourcePath = join(fixtures, 'deps/noir-script/src/main.nr');
export const depsScriptExpectedArtifact = join(fixtures, 'deps/noir-script/target/noir_wasm_testing.json');

export const libASourcePath = join(fixtures, 'deps/lib-a/src/lib.nr');
export const libBSourcePath = join(fixtures, 'deps/lib-b/src/lib.nr');
