import { parentPort } from 'worker_threads';
import { NodeListener, TransportServer } from '../../transport/index.js';
if (!parentPort) {
    throw new Error('InvalidWorker');
}
/**
 * Start the transport server corresponding to this module.
 * @param module - The WasmModule to host.
 */
export function startNodeModule(module) {
    const dispatch = async ({ fn, args }) => {
        if (fn === '__destroyWorker__') {
            transportServer.stop();
            return;
        }
        if (!module[fn]) {
            throw new Error(`dispatch error, function not found: ${fn}`);
        }
        return await module[fn](...args);
    };
    const transportListener = new NodeListener();
    const transportServer = new TransportServer(transportListener, dispatch);
    module.addLogger((...args) => transportServer.broadcast({ fn: 'emit', args: ['log', ...args] }));
    transportServer.start();
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic3RhcnRfbm9kZV9tb2R1bGUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvd29ya2VyL25vZGUvc3RhcnRfbm9kZV9tb2R1bGUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFFLFVBQVUsRUFBRSxNQUFNLGdCQUFnQixDQUFDO0FBQzVDLE9BQU8sRUFBRSxZQUFZLEVBQWUsZUFBZSxFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFHdEYsSUFBSSxDQUFDLFVBQVUsRUFBRTtJQUNmLE1BQU0sSUFBSSxLQUFLLENBQUMsZUFBZSxDQUFDLENBQUM7Q0FDbEM7QUFFRDs7O0dBR0c7QUFDSCxNQUFNLFVBQVUsZUFBZSxDQUFDLE1BQWtCO0lBQ2hELE1BQU0sUUFBUSxHQUFHLEtBQUssRUFBRSxFQUFFLEVBQUUsRUFBRSxJQUFJLEVBQWUsRUFBRSxFQUFFO1FBQ25ELElBQUksRUFBRSxLQUFLLG1CQUFtQixFQUFFO1lBQzlCLGVBQWUsQ0FBQyxJQUFJLEVBQUUsQ0FBQztZQUN2QixPQUFPO1NBQ1I7UUFDRCxJQUFJLENBQUUsTUFBYyxDQUFDLEVBQUUsQ0FBQyxFQUFFO1lBQ3hCLE1BQU0sSUFBSSxLQUFLLENBQUMsdUNBQXVDLEVBQUUsRUFBRSxDQUFDLENBQUM7U0FDOUQ7UUFDRCxPQUFPLE1BQU8sTUFBYyxDQUFDLEVBQUUsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLENBQUM7SUFDNUMsQ0FBQyxDQUFDO0lBQ0YsTUFBTSxpQkFBaUIsR0FBRyxJQUFJLFlBQVksRUFBRSxDQUFDO0lBQzdDLE1BQU0sZUFBZSxHQUFHLElBQUksZUFBZSxDQUFjLGlCQUFpQixFQUFFLFFBQVEsQ0FBQyxDQUFDO0lBQ3RGLE1BQU0sQ0FBQyxTQUFTLENBQUMsQ0FBQyxHQUFHLElBQVcsRUFBRSxFQUFFLENBQUMsZUFBZSxDQUFDLFNBQVMsQ0FBQyxFQUFFLEVBQUUsRUFBRSxNQUFNLEVBQUUsSUFBSSxFQUFFLENBQUMsS0FBSyxFQUFFLEdBQUcsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUM7SUFDeEcsZUFBZSxDQUFDLEtBQUssRUFBRSxDQUFDO0FBQzFCLENBQUMifQ==