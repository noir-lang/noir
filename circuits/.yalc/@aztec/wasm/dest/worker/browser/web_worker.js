import { WasmModule } from '../../wasm/wasm_module.js';
import { createDispatchProxy, TransportClient, WorkerConnector } from '../../transport/index.js';
/**
 * Instantiate a web worker.
 * @param url - The URL.
 * @param initialMem - Initial memory pages.
 * @param maxMem - Maximum memory pages.
 * @returns The worker.
 */
export async function createWebWorker(url, initialMem, maxMem) {
    const worker = new Worker(url);
    const transportConnect = new WorkerConnector(worker);
    const transportClient = new TransportClient(transportConnect);
    await transportClient.open();
    const remoteModule = createDispatchProxy(WasmModule, transportClient);
    remoteModule.destroyWorker = async () => {
        await transportClient.request({ fn: '__destroyWorker__', args: [] });
        transportClient.close();
    };
    await remoteModule.init(initialMem, maxMem);
    return remoteModule;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2ViX3dvcmtlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy93b3JrZXIvYnJvd3Nlci93ZWJfd29ya2VyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSwyQkFBMkIsQ0FBQztBQUN2RCxPQUFPLEVBQUUsbUJBQW1CLEVBQWUsZUFBZSxFQUFFLGVBQWUsRUFBRSxNQUFNLDBCQUEwQixDQUFDO0FBRzlHOzs7Ozs7R0FNRztBQUNILE1BQU0sQ0FBQyxLQUFLLFVBQVUsZUFBZSxDQUFDLEdBQVcsRUFBRSxVQUFtQixFQUFFLE1BQWU7SUFDckYsTUFBTSxNQUFNLEdBQUcsSUFBSSxNQUFNLENBQUMsR0FBRyxDQUFDLENBQUM7SUFDL0IsTUFBTSxnQkFBZ0IsR0FBRyxJQUFJLGVBQWUsQ0FBQyxNQUFNLENBQUMsQ0FBQztJQUNyRCxNQUFNLGVBQWUsR0FBRyxJQUFJLGVBQWUsQ0FBYyxnQkFBZ0IsQ0FBQyxDQUFDO0lBQzNFLE1BQU0sZUFBZSxDQUFDLElBQUksRUFBRSxDQUFDO0lBQzdCLE1BQU0sWUFBWSxHQUFHLG1CQUFtQixDQUFDLFVBQVUsRUFBRSxlQUFlLENBQWUsQ0FBQztJQUNwRixZQUFZLENBQUMsYUFBYSxHQUFHLEtBQUssSUFBSSxFQUFFO1FBQ3RDLE1BQU0sZUFBZSxDQUFDLE9BQU8sQ0FBQyxFQUFFLEVBQUUsRUFBRSxtQkFBbUIsRUFBRSxJQUFJLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FBQztRQUNyRSxlQUFlLENBQUMsS0FBSyxFQUFFLENBQUM7SUFDMUIsQ0FBQyxDQUFDO0lBQ0YsTUFBTSxZQUFZLENBQUMsSUFBSSxDQUFDLFVBQVUsRUFBRSxNQUFNLENBQUMsQ0FBQztJQUM1QyxPQUFPLFlBQVksQ0FBQztBQUN0QixDQUFDIn0=