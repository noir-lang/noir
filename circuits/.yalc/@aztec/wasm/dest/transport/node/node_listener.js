import { parentPort } from 'worker_threads';
import EventEmitter from 'events';
import { NodeListenerSocket } from './node_listener_socket.js';
/**
 * A socket listener that works with Node.
 */
export class NodeListener extends EventEmitter {
    constructor() {
        super();
    }
    /**
     * Open the listener.
     */
    open() {
        this.emit('new_socket', new NodeListenerSocket(parentPort));
    }
    /**
     * Close the listener.
     */
    close() { }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9saXN0ZW5lci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvbm9kZS9ub2RlX2xpc3RlbmVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSxnQkFBZ0IsQ0FBQztBQUM1QyxPQUFPLFlBQVksTUFBTSxRQUFRLENBQUM7QUFFbEMsT0FBTyxFQUFFLGtCQUFrQixFQUFFLE1BQU0sMkJBQTJCLENBQUM7QUFFL0Q7O0dBRUc7QUFDSCxNQUFNLE9BQU8sWUFBYSxTQUFRLFlBQVk7SUFDNUM7UUFDRSxLQUFLLEVBQUUsQ0FBQztJQUNWLENBQUM7SUFFRDs7T0FFRztJQUNILElBQUk7UUFDRixJQUFJLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxJQUFJLGtCQUFrQixDQUFDLFVBQWlCLENBQUMsQ0FBQyxDQUFDO0lBQ3JFLENBQUM7SUFFRDs7T0FFRztJQUNILEtBQUssS0FBSSxDQUFDO0NBQ1gifQ==