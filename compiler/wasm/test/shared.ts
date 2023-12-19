const currentPath = __dirname.split('/');

const fixtures = `/${currentPath.slice(0, currentPath.length - 1).join('/')}/public/fixtures`;

export const simpleScriptSourcePath = `${fixtures}/simple/src/main.nr`;
export const simpleScriptExpectedArtifact = `${fixtures}/simple/target/noir_wasm_testing.json`;

export const depsScriptSourcePath = `${fixtures}/with-deps/src/main.nr`;
export const depsScriptExpectedArtifact = `${fixtures}/with-deps/target/noir_wasm_testing.json`;

export const libASourcePath = `${fixtures}/deps/lib-a/src/lib.nr`;
export const libBSourcePath = `${fixtures}/deps/lib-b/src/lib.nr`;
