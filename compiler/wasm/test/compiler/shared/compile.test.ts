import { inflateDebugSymbols } from '@noir-lang/noir_wasm';
import { type expect as Expect } from 'chai';
import {
  ContractArtifact,
  ContractCompilationArtifacts,
  DebugFileMap,
  DebugInfo,
  ProgramDebugInfo,
  NoirFunctionEntry,
  ProgramArtifact,
  ProgramCompilationArtifacts,
} from '../../../src/types/noir_artifact';

export function shouldCompileProgramIdentically(
  compileFn: () => Promise<{ nargoArtifact: ProgramArtifact; noirWasmArtifact: ProgramCompilationArtifacts }>,
  expect: typeof Expect,
  timeout = 5000,
) {
  it('both nargo and noir_wasm should compile program identically', async () => {
    // Compile!
    const { nargoArtifact, noirWasmArtifact } = await compileFn();

    // Prepare nargo artifact
    const [_nargoDebugInfos, nargoFileMap] = deleteProgramDebugMetadata(nargoArtifact);
    normalizeVersion(nargoArtifact);

    // Prepare noir-wasm artifact
    const noirWasmProgram = noirWasmArtifact.program;
    expect(noirWasmProgram).not.to.be.undefined;
    const [_noirWasmDebugInfos, norWasmFileMap] = deleteProgramDebugMetadata(noirWasmProgram);
    normalizeVersion(noirWasmProgram);

    // We first compare both contracts without considering debug info
    delete (noirWasmProgram as Partial<ProgramArtifact>).hash;
    delete (nargoArtifact as Partial<ProgramArtifact>).hash;
    expect(nargoArtifact).to.deep.eq(noirWasmProgram);

    // Compare the file maps, ignoring keys, since those depend in the order in which files are visited,
    // which may change depending on the file manager implementation. Also ignores paths, since the base
    // path is reported differently between nargo and noir-wasm.
    expect(getSources(nargoFileMap)).to.have.members(getSources(norWasmFileMap));

    // Compare the debug symbol information, ignoring the actual ids used for file identifiers.
    // Debug symbol info looks like the following, what we need is to ignore the 'file' identifiers
    // {"locations":{"0":[{"span":{"start":141,"end":156},"file":39},{"span":{"start":38,"end":76},"file":38},{"span":{"start":824,"end":862},"file":23}]}}
    // expect(nargoDebugInfos).to.deep.eq(noirWasmDebugInfos);
  }).timeout(timeout);
}

export function shouldCompileContractIdentically(
  compileFn: () => Promise<{ nargoArtifact: ContractArtifact; noirWasmArtifact: ContractCompilationArtifacts }>,
  expect: typeof Expect,
  timeout = 5000,
) {
  it('both nargo and noir_wasm should compile contract identically', async () => {
    // Compile!
    const { nargoArtifact, noirWasmArtifact } = await compileFn();

    // Prepare nargo artifact
    const [nargoDebugInfos, nargoFileMap] = deleteContractDebugMetadata(nargoArtifact);
    normalizeVersion(nargoArtifact);

    // Prepare noir-wasm artifact
    const noirWasmContract = noirWasmArtifact.contract;
    expect(noirWasmContract).not.to.be.undefined;
    const [noirWasmDebugInfos, norWasmFileMap] = deleteContractDebugMetadata(noirWasmContract);
    normalizeVersion(noirWasmContract);

    // We first compare both contracts without considering debug info
    expect(nargoArtifact).to.deep.eq(noirWasmContract);

    // Compare the file maps, ignoring keys, since those depend in the order in which files are visited,
    // which may change depending on the file manager implementation. Also ignores paths, since the base
    // path is reported differently between nargo and noir-wasm.
    expect(getSources(nargoFileMap)).to.have.members(getSources(norWasmFileMap));

    // Compare the debug symbol information, ignoring the actual ids used for file identifiers.
    // Debug symbol info looks like the following, what we need is to ignore the 'file' identifiers
    // {"locations":{"0":[{"span":{"start":141,"end":156},"file":39},{"span":{"start":38,"end":76},"file":38},{"span":{"start":824,"end":862},"file":23}]}}
    expect(nargoDebugInfos).to.deep.eq(noirWasmDebugInfos);
  }).timeout(timeout);
}

/** Remove commit identifier from version, which may not match depending on cached nargo and noir-wasm */
function normalizeVersion(contract: ProgramArtifact | ContractArtifact) {
  contract.noir_version = contract.noir_version.replace(/\+.+$/, '');
}

/** Extracts the debug symbols from all functions, decodes them, removes their file identifiers, and deletes them from the artifact. */
function extractDebugInfos(fns: NoirFunctionEntry[]) {
  return fns.map((fn) => {
    const debugSymbols = inflateDebugSymbols(fn.debug_symbols);
    delete (fn as Partial<NoirFunctionEntry>).debug_symbols;
    clearFileIdentifiersProgram(debugSymbols);
    return debugSymbols;
  });
}

/** Deletes all debug info from a program and returns it. */
function deleteProgramDebugMetadata(program: ProgramArtifact) {
  const debugSymbols = inflateDebugSymbols(program.debug_symbols);
  const fileMap = program.file_map;

  delete (program as Partial<ProgramArtifact>).debug_symbols;
  delete (program as Partial<ProgramArtifact>).file_map;
  return [debugSymbols, fileMap];
}

/** Deletes all debug info from a contract and returns it. */
function deleteContractDebugMetadata(contract: ContractArtifact) {
  contract.functions.sort((a, b) => a.name.localeCompare(b.name));
  const fileMap = contract.file_map;
  delete (contract as Partial<ContractArtifact>).file_map;
  return [extractDebugInfos(contract.functions), fileMap];
}

function clearFileIdentifiersProgram(debugSymbols: ProgramDebugInfo) {
  debugSymbols.debug_infos.map((debug_info) => {
    clearFileIdentifiers(debug_info);
  });
}

/** Clears file identifiers from a set of debug symbols. */
function clearFileIdentifiers(debugSymbols: DebugInfo) {
  for (const locationNode of debugSymbols.location_tree.locations) {
    locationNode.value.file = 0;
  }
}

/** Returns list of sources from file map, dropping paths along the way, since they don't match depending on the file manager. */
function getSources(fileMap: DebugFileMap) {
  return Object.values(fileMap).map((file) => file.source);
}
