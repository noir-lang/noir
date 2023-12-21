export function getPaths(basePath: string) {
  const fixtures = `${basePath}/circuits`;

  const libProjectPath = `${fixtures}/lib_with_module`;
  const libSourcePath = `${libProjectPath}/src/lib.nr`;
  const libModulePath = `${libProjectPath}/src/module.nr`;
  const libModuleSourcePath = `${libProjectPath}/src/module/foo.nr`;
  const libTOMLPath = `${libProjectPath}/Nargo.toml`;

  const contractProjectPath = `${fixtures}/deps_testing`;
  const contractSourcePath = `${contractProjectPath}/src/main.nr`;
  const contractTOMLPath = `${contractProjectPath}/Nargo.toml`;
  const contractExpectedArtifact = `${contractProjectPath}/target/test-TestContract.json`;

  return {
    libProjectPath,
    libSourcePath,
    libModulePath,
    libModuleSourcePath,
    libTOMLPath,
    contractProjectPath,
    contractSourcePath,
    contractTOMLPath,
    contractExpectedArtifact,
  };
}
