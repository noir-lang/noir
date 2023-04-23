import { WasmModule } from '@aztec/foundation/wasm';
import { Crs } from '../crs/index.js';

/**
 * Loads the verifier CRS into WASM memory and returns a pointer to it.
 * @param wasm - WASM module.
 * @returns A pointer to the verifier CRS in WASM memory.
 */
export async function loadVerifierCrs(wasm: WasmModule) {
  // TODO optimize
  const crs = new Crs(0);
  await crs.init();
  const crsPtr = wasm.call('bbmalloc', crs.getG2Data().length);
  wasm.writeMemory(crsPtr, crs.getG2Data());
  return crsPtr;
}

/**
 * Loads the prover CRS into WASM memory and returns a pointer to it.
 * @param wasm - WASM module.
 * @param numPoints - The number of circuit gates.
 * @returns A pointer to the prover CRS in WASM memory.
 */
export async function loadProverCrs(wasm: WasmModule, numPoints: number) {
  const crs = new Crs(numPoints);
  await crs.init();
  const crsPtr = wasm.call('bbmalloc', crs.getG1Data().length);
  wasm.writeMemory(crsPtr, crs.getG1Data());
  return crsPtr;
}
