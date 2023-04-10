import { WasmModule } from '@aztec/foundation/wasm';
import { Crs } from '../crs/index.js';

export async function loadVerifierCrs(wasm: WasmModule) {
  // TODO optimize
  const crs = new Crs(0);
  await crs.init();
  const crsPtr = wasm.call('bbmalloc', crs.getG2Data().length);
  wasm.writeMemory(crsPtr, crs.getG2Data());
  return crsPtr;
}

export async function loadProverCrs(wasm: WasmModule, numPoints: number) {
  const crs = new Crs(numPoints);
  await crs.init();
  const crsPtr = wasm.call('bbmalloc', crs.getG1Data().length);
  wasm.writeMemory(crsPtr, crs.getG1Data());
  return crsPtr;
}
