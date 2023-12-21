export function getPaths(basePath: string) {
  const fixtures = `${basePath}/fixtures`;

  const simpleScriptSourcePath = `${fixtures}/simple/src/main.nr`;
  const simpleScriptExpectedArtifact = `${fixtures}/simple/target/noir_wasm_testing.json`;

  const depsScriptSourcePath = `${fixtures}/with-deps/src/main.nr`;
  const depsScriptExpectedArtifact = `${fixtures}/with-deps/target/noir_wasm_testing.json`;

  const libASourcePath = `${fixtures}/deps/lib-a/src/lib.nr`;
  const libBSourcePath = `${fixtures}/deps/lib-b/src/lib.nr`;

  return {
    simpleScriptSourcePath,
    simpleScriptExpectedArtifact,
    depsScriptSourcePath,
    depsScriptExpectedArtifact,
    libASourcePath,
    libBSourcePath,
  };
}
