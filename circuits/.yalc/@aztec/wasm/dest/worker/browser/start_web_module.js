import { TransportServer, WorkerListener } from '../../transport/index.js';
/**
 * Start the transport server corresponding to this module.
 * @param module - The WasmModule to host.
 */
export function startWebModule(module) {
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
    const transportListener = new WorkerListener(self);
    const transportServer = new TransportServer(transportListener, dispatch);
    module.addLogger((...args) => transportServer.broadcast({ fn: 'emit', args: ['log', ...args] }));
    transportServer.start();
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic3RhcnRfd2ViX21vZHVsZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy93b3JrZXIvYnJvd3Nlci9zdGFydF93ZWJfbW9kdWxlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBZSxlQUFlLEVBQUUsY0FBYyxFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFHeEY7OztHQUdHO0FBQ0gsTUFBTSxVQUFVLGNBQWMsQ0FBQyxNQUFrQjtJQUMvQyxNQUFNLFFBQVEsR0FBRyxLQUFLLEVBQUUsRUFBRSxFQUFFLEVBQUUsSUFBSSxFQUFlLEVBQUUsRUFBRTtRQUNuRCxJQUFJLEVBQUUsS0FBSyxtQkFBbUIsRUFBRTtZQUM5QixlQUFlLENBQUMsSUFBSSxFQUFFLENBQUM7WUFDdkIsT0FBTztTQUNSO1FBQ0QsSUFBSSxDQUFFLE1BQWMsQ0FBQyxFQUFFLENBQUMsRUFBRTtZQUN4QixNQUFNLElBQUksS0FBSyxDQUFDLHVDQUF1QyxFQUFFLEVBQUUsQ0FBQyxDQUFDO1NBQzlEO1FBQ0QsT0FBTyxNQUFPLE1BQWMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxDQUFDO0lBQzVDLENBQUMsQ0FBQztJQUNGLE1BQU0saUJBQWlCLEdBQUcsSUFBSSxjQUFjLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDbkQsTUFBTSxlQUFlLEdBQUcsSUFBSSxlQUFlLENBQWMsaUJBQWlCLEVBQUUsUUFBUSxDQUFDLENBQUM7SUFDdEYsTUFBTSxDQUFDLFNBQVMsQ0FBQyxDQUFDLEdBQUcsSUFBVyxFQUFFLEVBQUUsQ0FBQyxlQUFlLENBQUMsU0FBUyxDQUFDLEVBQUUsRUFBRSxFQUFFLE1BQU0sRUFBRSxJQUFJLEVBQUUsQ0FBQyxLQUFLLEVBQUUsR0FBRyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQztJQUN4RyxlQUFlLENBQUMsS0FBSyxFQUFFLENBQUM7QUFDMUIsQ0FBQyJ9