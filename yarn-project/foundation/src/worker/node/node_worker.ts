import { Worker } from 'worker_threads';

import { type DispatchMsg, NodeConnector, TransportClient, createDispatchProxy } from '../../transport/index.js';
import { WasmModule } from '../../wasm/wasm_module.js';
import { type WasmWorker } from '../wasm_worker.js';

/**
 * Creates a node worker.
 */
export async function createNodeWorker(filepath: string, initialMem?: number, maxMem?: number): Promise<WasmWorker> {
  const worker = new Worker(filepath);
  const transportConnect = new NodeConnector(worker);
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
