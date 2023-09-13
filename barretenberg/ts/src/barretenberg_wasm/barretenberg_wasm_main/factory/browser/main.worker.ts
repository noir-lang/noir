import { expose } from 'comlink';
import { BarretenbergWasmMain } from '../../index.js';
import debug from 'debug';

self.onmessage = function (e) {
  if (e.data.debug) {
    debug.enable(e.data.debug);
  }
};

expose(new BarretenbergWasmMain());

self.postMessage({ ready: true });

export default null as any;
