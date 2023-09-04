import { Worker } from 'worker_threads';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

export function createMainWorker() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return new Worker(__dirname + `/main.worker.js`);
}
