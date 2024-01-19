import { CompilationResult, inflateDebugSymbols } from '@noir-lang/noir_wasm';
import { type expect as Expect } from 'chai';
import {
  ContractArtifact,
  ContractCompilationArtifacts,
  DebugFileMap,
  DebugInfo,
  NoirFunctionEntry,
} from '../../../src/types/noir_artifact';

export function shouldCompileIdentically(
  compileFn: () => Promise<{ nargoArtifact: ContractArtifact; noirWasmArtifact: CompilationResult }>,
  expect: typeof Expect,
  timeout = 5000,
) {
  it('both nargo and noir_wasm should compile identically', async () => {
    // Compile!
    const { nargoArtifact, noirWasmArtifact } = await compileFn();

    // Prepare nargo artifact
    const [nargoDebugInfos, nargoFileMap] = deleteDebugMetadata(nargoArtifact);
    normalizeVersion(nargoArtifact);

    // Prepare noir-wasm artifact
    const noirWasmContract = (noirWasmArtifact as ContractCompilationArtifacts).contract;
    expect(noirWasmContract).not.to.be.undefined;
    const [noirWasmDebugInfos, norWasmFileMap] = deleteDebugMetadata(noirWasmContract);
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
function normalizeVersion(contract: ContractArtifact) {
  contract.noir_version = contract.noir_version.replace(/\+.+$/, '');
}

/** Extracts the debug symbols from all functions, decodes them, removes their file identifiers, and deletes them from the artifact. */
function extractDebugInfos(fns: NoirFunctionEntry[]) {
  return fns.map((fn) => {
    const debugSymbols = inflateDebugSymbols(fn.debug_symbols);
    delete (fn as Partial<NoirFunctionEntry>).debug_symbols;
    clearFileIdentifiers(debugSymbols);
    return debugSymbols;
  });
}

/** Deletes all debug info from a contract and returns it. */
function deleteDebugMetadata(contract: ContractArtifact) {
  contract.functions.sort((a, b) => a.name.localeCompare(b.name));
  const fileMap = contract.file_map;
  delete (contract as Partial<ContractArtifact>).file_map;
  return [extractDebugInfos(contract.functions), fileMap];
}

/** Clears file identifiers from a set of debug symbols. */
function clearFileIdentifiers(debugSymbols: DebugInfo) {
  for (const loc of Object.values(debugSymbols.locations)) {
    for (const span of loc) {
      span.file = 0;
    }
  }
}

/** Returns list of sources from file map, dropping paths along the way, since they don't match depending on the file manager. */
function getSources(fileMap: DebugFileMap) {
  return Object.values(fileMap).map((file) => file.source);
}
