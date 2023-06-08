import { parentPort } from 'worker_threads';
import { expose } from 'comlink';
import { BarretenbergWasm } from '../index.js';
import { nodeEndpoint } from './node_endpoint.js';

if (!parentPort) {
  throw new Error('No parentPort');
}

expose(new BarretenbergWasm(), nodeEndpoint(parentPort));
