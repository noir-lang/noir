import EventEmitter from 'events';
import { MessagePortSocket } from './message_port_socket.js';
/**
 *
 */
export class WorkerListener extends EventEmitter {
    /**
     *
     * @param worker
     */
    constructor(worker) {
        super();
        this.worker = worker;
        /**
         *
         * @param event
         */
        this.handleMessageEvent = (event) => {
            const [port] = event.ports;
            if (!port) {
                return;
            }
            this.emit('new_socket', new MessagePortSocket(port));
        };
    }
    /**
     *
     */
    open() {
        this.worker.onmessage = this.handleMessageEvent;
    }
    /**
     *
     */
    close() {
        this.worker.onmessage = () => { };
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid29ya2VyX2xpc3RlbmVyLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9icm93c2VyL3dvcmtlcl9saXN0ZW5lci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLFlBQVksTUFBTSxRQUFRLENBQUM7QUFFbEMsT0FBTyxFQUFFLGlCQUFpQixFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFZN0Q7O0dBRUc7QUFDSCxNQUFNLE9BQU8sY0FBZSxTQUFRLFlBQVk7SUFDOUM7OztPQUdHO0lBQ0gsWUFBb0IsTUFBa0M7UUFDcEQsS0FBSyxFQUFFLENBQUM7UUFEVSxXQUFNLEdBQU4sTUFBTSxDQUE0QjtRQWtCdEQ7OztXQUdHO1FBQ0ssdUJBQWtCLEdBQUcsQ0FBQyxLQUFtQixFQUFFLEVBQUU7WUFDbkQsTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLEtBQUssQ0FBQyxLQUFLLENBQUM7WUFDM0IsSUFBSSxDQUFDLElBQUksRUFBRTtnQkFDVCxPQUFPO2FBQ1I7WUFDRCxJQUFJLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxJQUFJLGlCQUFpQixDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7UUFDdkQsQ0FBQyxDQUFDO0lBMUJGLENBQUM7SUFFRDs7T0FFRztJQUNILElBQUk7UUFDRixJQUFJLENBQUMsTUFBTSxDQUFDLFNBQVMsR0FBRyxJQUFJLENBQUMsa0JBQWtCLENBQUM7SUFDbEQsQ0FBQztJQUVEOztPQUVHO0lBQ0gsS0FBSztRQUNILElBQUksQ0FBQyxNQUFNLENBQUMsU0FBUyxHQUFHLEdBQUcsRUFBRSxHQUFFLENBQUMsQ0FBQztJQUNuQyxDQUFDO0NBYUYifQ==