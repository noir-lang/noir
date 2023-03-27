import { Worker } from 'worker_threads';
import { createDispatchProxy, TransportClient } from '../../transport/index.js';
import { NodeConnector } from '../../transport/index.js';
import { WasmModule } from '../../wasm/wasm_module.js';
/**
 *
 */
export async function createNodeWorker(filepath, initialMem, maxMem) {
    const worker = new Worker(filepath);
    const transportConnect = new NodeConnector(worker);
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV93b3JrZXIuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvd29ya2VyL25vZGUvbm9kZV93b3JrZXIudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLGdCQUFnQixDQUFDO0FBQ3hDLE9BQU8sRUFBRSxtQkFBbUIsRUFBZSxlQUFlLEVBQUUsTUFBTSwwQkFBMEIsQ0FBQztBQUM3RixPQUFPLEVBQUUsYUFBYSxFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFDekQsT0FBTyxFQUFFLFVBQVUsRUFBRSxNQUFNLDJCQUEyQixDQUFDO0FBR3ZEOztHQUVHO0FBQ0gsTUFBTSxDQUFDLEtBQUssVUFBVSxnQkFBZ0IsQ0FBQyxRQUFnQixFQUFFLFVBQW1CLEVBQUUsTUFBZTtJQUMzRixNQUFNLE1BQU0sR0FBRyxJQUFJLE1BQU0sQ0FBQyxRQUFRLENBQUMsQ0FBQztJQUNwQyxNQUFNLGdCQUFnQixHQUFHLElBQUksYUFBYSxDQUFDLE1BQU0sQ0FBQyxDQUFDO0lBQ25ELE1BQU0sZUFBZSxHQUFHLElBQUksZUFBZSxDQUFjLGdCQUFnQixDQUFDLENBQUM7SUFDM0UsTUFBTSxlQUFlLENBQUMsSUFBSSxFQUFFLENBQUM7SUFDN0IsTUFBTSxZQUFZLEdBQUcsbUJBQW1CLENBQUMsVUFBVSxFQUFFLGVBQWUsQ0FBZSxDQUFDO0lBQ3BGLFlBQVksQ0FBQyxhQUFhLEdBQUcsS0FBSyxJQUFJLEVBQUU7UUFDdEMsTUFBTSxlQUFlLENBQUMsT0FBTyxDQUFDLEVBQUUsRUFBRSxFQUFFLG1CQUFtQixFQUFFLElBQUksRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQ3JFLGVBQWUsQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUMxQixDQUFDLENBQUM7SUFDRixNQUFNLFlBQVksQ0FBQyxJQUFJLENBQUMsVUFBVSxFQUFFLE1BQU0sQ0FBQyxDQUFDO0lBQzVDLE9BQU8sWUFBWSxDQUFDO0FBQ3RCLENBQUMifQ==