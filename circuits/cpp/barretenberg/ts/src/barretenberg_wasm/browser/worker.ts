import { expose } from 'comlink';
import { BarretenbergWasm } from '../index.js';
import debug from 'debug';

self.onmessage = function (e) {
  if (e.data.debug) {
    debug.enable(e.data.debug);
  }
};

expose(new BarretenbergWasm());

self.postMessage({ ready: true });
