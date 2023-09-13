import { parentPort } from 'worker_threads';
import { expose } from 'comlink';
import { BarretenbergWasmThread } from '../../index.js';
import { nodeEndpoint } from '../../../helpers/node/node_endpoint.js';

if (!parentPort) {
  throw new Error('No parentPort');
}

expose(new BarretenbergWasmThread(), nodeEndpoint(parentPort));
