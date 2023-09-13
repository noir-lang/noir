import debug from 'debug';
import MainWorker from './main.worker.js';

export function createMainWorker() {
  const worker = new MainWorker();
  const debugStr = debug.disable();
  debug.enable(debugStr);
  worker.postMessage({ debug: debugStr });
  return worker;
}
