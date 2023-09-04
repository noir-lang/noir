import { wrap } from 'comlink';

export function getSharedMemoryAvailable() {
  const globalScope = typeof window !== 'undefined' ? window : self;
  return typeof SharedArrayBuffer !== 'undefined' && globalScope.crossOriginIsolated;
}

export function getRemoteBarretenbergWasm<T>(worker: Worker) {
  return wrap(worker) as T;
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
