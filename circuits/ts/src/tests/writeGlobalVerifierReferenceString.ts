import { Crs } from '../crs/index.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

/**
 * Write to the global VRS in C++.
 * @param circuitSize The circuit size.
 */

export async function writeGlobalVerifierReferenceString(wasm: CircuitsWasm, circuitSize: number) {
  const crs: Crs = new Crs(/*example, circuit size = 100*/ 100);
  await crs.init();
  const g2DataPtr = wasm.call('bbmalloc', crs.getG2Data().length);
  wasm.writeMemory(g2DataPtr, crs.getG2Data());
  wasm.call('abis__set_global_verifier_reference_string', g2DataPtr);
  wasm.call('bbfree', g2DataPtr);
}
