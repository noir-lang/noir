import { parentPort } from 'worker_threads';
import EventEmitter from 'events';
import { NodeListenerSocket } from './node_listener_socket.js';
export class NodeListener extends EventEmitter {
    constructor() {
        super();
    }
    open() {
        this.emit('new_socket', new NodeListenerSocket(parentPort));
    }
    close() { }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9saXN0ZW5lci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvbm9kZS9ub2RlX2xpc3RlbmVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxVQUFVLEVBQUUsTUFBTSxnQkFBZ0IsQ0FBQztBQUM1QyxPQUFPLFlBQVksTUFBTSxRQUFRLENBQUM7QUFFbEMsT0FBTyxFQUFFLGtCQUFrQixFQUFFLE1BQU0sMkJBQTJCLENBQUM7QUFFL0QsTUFBTSxPQUFPLFlBQWEsU0FBUSxZQUFZO0lBQzVDO1FBQ0UsS0FBSyxFQUFFLENBQUM7SUFDVixDQUFDO0lBRUQsSUFBSTtRQUNGLElBQUksQ0FBQyxJQUFJLENBQUMsWUFBWSxFQUFFLElBQUksa0JBQWtCLENBQUMsVUFBaUIsQ0FBQyxDQUFDLENBQUM7SUFDckUsQ0FBQztJQUVELEtBQUssS0FBSSxDQUFDO0NBQ1gifQ==