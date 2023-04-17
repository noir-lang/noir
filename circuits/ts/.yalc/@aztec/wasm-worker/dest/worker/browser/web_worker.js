import { WasmModule } from '../../wasm/wasm_module.js';
import { createDispatchProxy, TransportClient, WorkerConnector } from '../../transport/index.js';
/**
 *
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2ViX3dvcmtlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy93b3JrZXIvYnJvd3Nlci93ZWJfd29ya2VyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSwyQkFBMkIsQ0FBQztBQUN2RCxPQUFPLEVBQUUsbUJBQW1CLEVBQWUsZUFBZSxFQUFFLGVBQWUsRUFBRSxNQUFNLDBCQUEwQixDQUFDO0FBRzlHOztHQUVHO0FBQ0gsTUFBTSxDQUFDLEtBQUssVUFBVSxlQUFlLENBQUMsR0FBVyxFQUFFLFVBQW1CLEVBQUUsTUFBZTtJQUNyRixNQUFNLE1BQU0sR0FBRyxJQUFJLE1BQU0sQ0FBQyxHQUFHLENBQUMsQ0FBQztJQUMvQixNQUFNLGdCQUFnQixHQUFHLElBQUksZUFBZSxDQUFDLE1BQU0sQ0FBQyxDQUFDO0lBQ3JELE1BQU0sZUFBZSxHQUFHLElBQUksZUFBZSxDQUFjLGdCQUFnQixDQUFDLENBQUM7SUFDM0UsTUFBTSxlQUFlLENBQUMsSUFBSSxFQUFFLENBQUM7SUFDN0IsTUFBTSxZQUFZLEdBQUcsbUJBQW1CLENBQUMsVUFBVSxFQUFFLGVBQWUsQ0FBZSxDQUFDO0lBQ3BGLFlBQVksQ0FBQyxhQUFhLEdBQUcsS0FBSyxJQUFJLEVBQUU7UUFDdEMsTUFBTSxlQUFlLENBQUMsT0FBTyxDQUFDLEVBQUUsRUFBRSxFQUFFLG1CQUFtQixFQUFFLElBQUksRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQ3JFLGVBQWUsQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUMxQixDQUFDLENBQUM7SUFDRixNQUFNLFlBQVksQ0FBQyxJQUFJLENBQUMsVUFBVSxFQUFFLE1BQU0sQ0FBQyxDQUFDO0lBQzVDLE9BQU8sWUFBWSxDQUFDO0FBQ3RCLENBQUMifQ==