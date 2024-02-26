export function getPaths(basePath: string) {
  const fixtures = `${basePath}/fixtures`;

  const simpleScriptProjectPath = `${fixtures}/simple`;
  const simpleScriptSourcePath = `${simpleScriptProjectPath}/src/main.nr`;
  const simpleScriptTOMLPath = `${simpleScriptProjectPath}/Nargo.toml`;
  const simpleScriptExpectedArtifact = `${simpleScriptProjectPath}/target/noir_wasm_testing.json`;

  const depsScriptProjectPath = `${fixtures}/with-deps`;
  const depsScriptSourcePath = `${depsScriptProjectPath}/src/main.nr`;
  const depsScriptTOMLPath = `${depsScriptProjectPath}/Nargo.toml`;
  const depsScriptExpectedArtifact = `${depsScriptProjectPath}/target/noir_wasm_testing.json`;

  const libAProjectPath = `${fixtures}/deps/lib-a`;
  const libASourcePath = `${libAProjectPath}/src/lib.nr`;
  const libATOMLPath = `${libAProjectPath}/Nargo.toml`;

  const libBProjectPath = `${fixtures}/deps/lib-b`;
  const libBSourcePath = `${libBProjectPath}/src/lib.nr`;
  const libBTOMLPath = `${libBProjectPath}/Nargo.toml`;

  const contractProjectPath = `${fixtures}/noir-contract`;
  const contractSourcePath = `${contractProjectPath}/src/main.nr`;
  const contractTOMLPath = `${contractProjectPath}/Nargo.toml`;
  const contractExpectedArtifact = `${contractProjectPath}/target/test-TestContract.json`;

  const libCProjectPath = `${fixtures}/deps/lib-c`;
  const libCSourcePath = `${libCProjectPath}/src/lib.nr`;
  const libCModulePath = `${libCProjectPath}/src/module.nr`;
  const libCModuleSourcePath = `${libCProjectPath}/src/module/foo.nr`;
  const libCTOMLPath = `${libCProjectPath}/Nargo.toml`;

  return {
    simpleScriptProjectPath,
    simpleScriptSourcePath,
    simpleScriptTOMLPath,
    simpleScriptExpectedArtifact,
    depsScriptProjectPath,
    depsScriptSourcePath,
    depsScriptTOMLPath,
    depsScriptExpectedArtifact,
    libASourcePath,
    libATOMLPath,
    libBSourcePath,
    libBTOMLPath,
    contractProjectPath,
    contractSourcePath,
    contractTOMLPath,
    contractExpectedArtifact,
    libCProjectPath,
    libCSourcePath,
    libCModulePath,
    libCModuleSourcePath,
    libCTOMLPath,
  };
}
