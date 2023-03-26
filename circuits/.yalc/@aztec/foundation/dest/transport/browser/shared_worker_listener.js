import EventEmitter from 'events';
import { MessagePortSocket } from './message_port_socket.js';
export class SharedWorkerListener extends EventEmitter {
    constructor(worker) {
        super();
        this.worker = worker;
        this.handleMessageEvent = (event) => {
            const [port] = event.ports;
            if (!port) {
                return;
            }
            this.emit('new_socket', new MessagePortSocket(port));
        };
    }
    open() {
        this.worker.onconnect = this.handleMessageEvent;
    }
    close() {
        this.worker.onconnect = () => { };
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2hhcmVkX3dvcmtlcl9saXN0ZW5lci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci9zaGFyZWRfd29ya2VyX2xpc3RlbmVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sWUFBWSxNQUFNLFFBQVEsQ0FBQztBQUVsQyxPQUFPLEVBQUUsaUJBQWlCLEVBQUUsTUFBTSwwQkFBMEIsQ0FBQztBQU03RCxNQUFNLE9BQU8sb0JBQXFCLFNBQVEsWUFBWTtJQUNwRCxZQUFvQixNQUErQjtRQUNqRCxLQUFLLEVBQUUsQ0FBQztRQURVLFdBQU0sR0FBTixNQUFNLENBQXlCO1FBWTNDLHVCQUFrQixHQUFHLENBQUMsS0FBbUIsRUFBRSxFQUFFO1lBQ25ELE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxLQUFLLENBQUMsS0FBSyxDQUFDO1lBQzNCLElBQUksQ0FBQyxJQUFJLEVBQUU7Z0JBQ1QsT0FBTzthQUNSO1lBQ0QsSUFBSSxDQUFDLElBQUksQ0FBQyxZQUFZLEVBQUUsSUFBSSxpQkFBaUIsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDO1FBQ3ZELENBQUMsQ0FBQztJQWhCRixDQUFDO0lBRUQsSUFBSTtRQUNGLElBQUksQ0FBQyxNQUFNLENBQUMsU0FBUyxHQUFHLElBQUksQ0FBQyxrQkFBa0IsQ0FBQztJQUNsRCxDQUFDO0lBRUQsS0FBSztRQUNILElBQUksQ0FBQyxNQUFNLENBQUMsU0FBUyxHQUFHLEdBQUcsRUFBRSxHQUFFLENBQUMsQ0FBQztJQUNuQyxDQUFDO0NBU0YifQ==