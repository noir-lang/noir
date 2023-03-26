import { MessagePortSocket } from './message_port_socket.js';
export class WorkerConnector {
    constructor(worker) {
        this.worker = worker;
    }
    createSocket() {
        const channel = new MessageChannel();
        this.worker.postMessage('', [channel.port2]);
        return Promise.resolve(new MessagePortSocket(channel.port1));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid29ya2VyX2Nvbm5lY3Rvci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci93b3JrZXJfY29ubmVjdG9yLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUNBLE9BQU8sRUFBRSxpQkFBaUIsRUFBRSxNQUFNLDBCQUEwQixDQUFDO0FBRTdELE1BQU0sT0FBTyxlQUFlO0lBQzFCLFlBQW9CLE1BQWM7UUFBZCxXQUFNLEdBQU4sTUFBTSxDQUFRO0lBQUcsQ0FBQztJQUV0QyxZQUFZO1FBQ1YsTUFBTSxPQUFPLEdBQUcsSUFBSSxjQUFjLEVBQUUsQ0FBQztRQUNyQyxJQUFJLENBQUMsTUFBTSxDQUFDLFdBQVcsQ0FBQyxFQUFFLEVBQUUsQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQztRQUM3QyxPQUFPLE9BQU8sQ0FBQyxPQUFPLENBQUMsSUFBSSxpQkFBaUIsQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQztJQUMvRCxDQUFDO0NBQ0YifQ==