import { wrap } from 'comlink';
import { BarretenbergWasmWorker, type BarretenbergWasm } from '../barretenberg_wasm.js';
import debug from 'debug';

export async function fetchCode(name: string) {
  const res = await fetch('/' + name);
  return await res.arrayBuffer();
}

export function createWorker() {
  const worker = new Worker('barretenberg_wasm.js');
  const debugStr = debug.disable();
  debug.enable(debugStr);
  worker.postMessage({ debug: debugStr });
  return worker;
}

export function getRemoteBarretenbergWasm(worker: Worker): BarretenbergWasmWorker {
  return wrap<BarretenbergWasm>(worker);
}

export function getNumCpu() {
  return navigator.hardwareConcurrency;
}

export function threadLogger(): ((msg: string) => void) | undefined {
  return undefined;
}

export function killSelf() {
  self.close();
}
