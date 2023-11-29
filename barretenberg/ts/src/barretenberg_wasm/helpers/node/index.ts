import { Worker } from 'worker_threads';
import os from 'os';
import { wrap } from 'comlink';
import { nodeEndpoint } from './node_endpoint.js';
import { writeSync } from 'fs';

export function getSharedMemoryAvailable() {
  return true;
}

/**
 * Comlink allows you to produce a Proxy to the worker, enabling you to call methods as if it were a normal class.
 * Note we give it the type information it needs so the returned Proxy object looks like that type.
 * Node has a different implementation, needing this nodeEndpoint wrapper, hence this function exists here.
 */
export function getRemoteBarretenbergWasm<T>(worker: Worker): T {
  return wrap(nodeEndpoint(worker)) as T;
}

/**
 * Returns number of cpus as reported by the system, unless overriden by HARDWARE_CONCURRENCY env var.
 */
export function getNumCpu() {
  return +process.env.HARDWARE_CONCURRENCY! || os.cpus().length;
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
