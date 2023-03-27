import EventEmitter from 'events';
import { MessagePortSocket } from './message_port_socket.js';
/**
 * Listens for connections to a shared worker.
 */
export class SharedWorkerListener extends EventEmitter {
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
        this.worker.onconnect = this.handleMessageEvent;
    }
    /**
     *
     */
    close() {
        this.worker.onconnect = () => { };
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2hhcmVkX3dvcmtlcl9saXN0ZW5lci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci9zaGFyZWRfd29ya2VyX2xpc3RlbmVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sWUFBWSxNQUFNLFFBQVEsQ0FBQztBQUVsQyxPQUFPLEVBQUUsaUJBQWlCLEVBQUUsTUFBTSwwQkFBMEIsQ0FBQztBQVk3RDs7R0FFRztBQUNILE1BQU0sT0FBTyxvQkFBcUIsU0FBUSxZQUFZO0lBQ3BEOzs7T0FHRztJQUNILFlBQW9CLE1BQStCO1FBQ2pELEtBQUssRUFBRSxDQUFDO1FBRFUsV0FBTSxHQUFOLE1BQU0sQ0FBeUI7UUFrQm5EOzs7V0FHRztRQUNLLHVCQUFrQixHQUFHLENBQUMsS0FBbUIsRUFBRSxFQUFFO1lBQ25ELE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxLQUFLLENBQUMsS0FBSyxDQUFDO1lBQzNCLElBQUksQ0FBQyxJQUFJLEVBQUU7Z0JBQ1QsT0FBTzthQUNSO1lBQ0QsSUFBSSxDQUFDLElBQUksQ0FBQyxZQUFZLEVBQUUsSUFBSSxpQkFBaUIsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDO1FBQ3ZELENBQUMsQ0FBQztJQTFCRixDQUFDO0lBRUQ7O09BRUc7SUFDSCxJQUFJO1FBQ0YsSUFBSSxDQUFDLE1BQU0sQ0FBQyxTQUFTLEdBQUcsSUFBSSxDQUFDLGtCQUFrQixDQUFDO0lBQ2xELENBQUM7SUFFRDs7T0FFRztJQUNILEtBQUs7UUFDSCxJQUFJLENBQUMsTUFBTSxDQUFDLFNBQVMsR0FBRyxHQUFHLEVBQUUsR0FBRSxDQUFDLENBQUM7SUFDbkMsQ0FBQztDQWFGIn0=