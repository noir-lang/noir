import debug from 'debug';
import ThreadWorker from './thread.worker.js';

export function createThreadWorker() {
  const worker = new ThreadWorker();
  const debugStr = debug.disable();
  debug.enable(debugStr);
  worker.postMessage({ debug: debugStr });
  return worker;
}
