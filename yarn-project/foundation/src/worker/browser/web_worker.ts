import { type DispatchMsg, TransportClient, WorkerConnector, createDispatchProxy } from '../../transport/index.js';
import { WasmModule } from '../../wasm/index.js';
import { type WasmWorker } from '../wasm_worker.js';

/**
 * Instantiate a web worker.
 * @param url - The URL.
 * @param initialMem - Initial memory pages.
 * @param maxMem - Maximum memory pages.
 * @returns The worker.
 */
export async function createWebWorker(url: string, initialMem?: number, maxMem?: number): Promise<WasmWorker> {
  const worker = new Worker(url);
  const transportConnect = new WorkerConnector(worker);
  const transportClient = new TransportClient<DispatchMsg>(transportConnect);
  await transportClient.open();
  const remoteModule = createDispatchProxy(WasmModule, transportClient) as WasmWorker;
  remoteModule.destroyWorker = async () => {
    await transportClient.request({ fn: '__destroyWorker__', args: [] });
    transportClient.close();
  };
  await remoteModule.init(initialMem, maxMem);
  return remoteModule;
}
