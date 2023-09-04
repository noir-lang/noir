import { parentPort } from 'worker_threads';
import { expose } from 'comlink';
import { BarretenbergWasmMain } from '../../index.js';
import { nodeEndpoint } from '../../../helpers/node/node_endpoint.js';

if (!parentPort) {
  throw new Error('No parentPort');
}

expose(new BarretenbergWasmMain(), nodeEndpoint(parentPort));
