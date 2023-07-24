import { Worker } from 'worker_threads';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { readFile } from 'fs/promises';
import os from 'os';
import { type BarretenbergWasm, type BarretenbergWasmWorker } from '../barretenberg_wasm.js';
import { wrap } from 'comlink';
import { nodeEndpoint } from './node_endpoint.js';
import { writeSync } from 'fs';

export async function fetchCode(multithreading: boolean) {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return await readFile(__dirname + `/../../${multithreading ? 'barretenberg-threads.wasm' : 'barretenberg.wasm'}`);
}

export function createWorker() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return new Worker(__dirname + `/worker.js`);
}

export function getRemoteBarretenbergWasm(worker: Worker): BarretenbergWasmWorker {
  return wrap<BarretenbergWasm>(nodeEndpoint(worker)) as BarretenbergWasmWorker;
}

export function getNumCpu() {
  return os.cpus().length;
}

/**
 * In node, the message passing is different to the browser. When using 'debug' in the browser, we seemingly always
 * get our logs, but in node it looks like it's dependent on the chain of workers from child to main thread be
 * unblocked. If one of our threads aborts, we can't see it as the parent is blocked waiting on threads to join.
 * To work around this in node, threads will by default write directly to stdout.
 */
export function threadLogger(): ((msg: string) => void) | undefined {
  return (msg: string) => {
    writeSync(1, msg + '\n');
  };
}

export function killSelf(): never {
  // Extordinarily hard process termination. Due to how parent threads block on child threads etc, even process.exit
  // doesn't seem to be able to abort the process. The following does.
  process.kill(process.pid);
  throw new Error();
}
