import EventEmitter from 'events';
import { MessagePortSocket } from './message_port_socket.js';
export class WorkerListener extends EventEmitter {
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
        this.worker.onmessage = this.handleMessageEvent;
    }
    close() {
        this.worker.onmessage = () => { };
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid29ya2VyX2xpc3RlbmVyLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9icm93c2VyL3dvcmtlcl9saXN0ZW5lci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLFlBQVksTUFBTSxRQUFRLENBQUM7QUFFbEMsT0FBTyxFQUFFLGlCQUFpQixFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFNN0QsTUFBTSxPQUFPLGNBQWUsU0FBUSxZQUFZO0lBQzlDLFlBQW9CLE1BQWtDO1FBQ3BELEtBQUssRUFBRSxDQUFDO1FBRFUsV0FBTSxHQUFOLE1BQU0sQ0FBNEI7UUFZOUMsdUJBQWtCLEdBQUcsQ0FBQyxLQUFtQixFQUFFLEVBQUU7WUFDbkQsTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLEtBQUssQ0FBQyxLQUFLLENBQUM7WUFDM0IsSUFBSSxDQUFDLElBQUksRUFBRTtnQkFDVCxPQUFPO2FBQ1I7WUFDRCxJQUFJLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxJQUFJLGlCQUFpQixDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7UUFDdkQsQ0FBQyxDQUFDO0lBaEJGLENBQUM7SUFFRCxJQUFJO1FBQ0YsSUFBSSxDQUFDLE1BQU0sQ0FBQyxTQUFTLEdBQUcsSUFBSSxDQUFDLGtCQUFrQixDQUFDO0lBQ2xELENBQUM7SUFFRCxLQUFLO1FBQ0gsSUFBSSxDQUFDLE1BQU0sQ0FBQyxTQUFTLEdBQUcsR0FBRyxFQUFFLEdBQUUsQ0FBQyxDQUFDO0lBQ25DLENBQUM7Q0FTRiJ9