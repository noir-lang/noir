import { type DispatchMsg, TransportServer, WorkerListener } from '../../transport/index.js';
import { type WasmModule } from '../../wasm/index.js';

/**
 * Start the transport server corresponding to this module.
 * @param module - The WasmModule to host.
 */
export function startWebModule(module: WasmModule) {
  const dispatch = async ({ fn, args }: DispatchMsg) => {
    if (fn === '__destroyWorker__') {
      transportServer.stop();
      return;
    }
    if (!(module as any)[fn]) {
      throw new Error(`dispatch error, function not found: ${fn}`);
    }
    return await (module as any)[fn](...args);
  };
  const transportListener = new WorkerListener(self);
  const transportServer = new TransportServer<DispatchMsg>(transportListener, dispatch);
  module.addLogger((...args: any[]) => transportServer.broadcast({ fn: 'emit', args: ['log', ...args] }));
  transportServer.start();
}
