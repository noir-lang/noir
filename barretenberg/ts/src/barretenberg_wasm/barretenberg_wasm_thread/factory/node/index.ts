import { Worker } from 'worker_threads';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

export function createThreadWorker() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return new Worker(__dirname + `/thread.worker.js`);
}
