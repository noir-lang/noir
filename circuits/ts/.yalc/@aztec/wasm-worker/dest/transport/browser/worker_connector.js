import { MessagePortSocket } from './message_port_socket.js';
/**
 *
 */
export class WorkerConnector {
    /**
     *
     * @param worker
     */
    constructor(worker) {
        this.worker = worker;
    }
    /**
     *
     */
    createSocket() {
        const channel = new MessageChannel();
        this.worker.postMessage('', [channel.port2]);
        return Promise.resolve(new MessagePortSocket(channel.port1));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid29ya2VyX2Nvbm5lY3Rvci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci93b3JrZXJfY29ubmVjdG9yLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUNBLE9BQU8sRUFBRSxpQkFBaUIsRUFBRSxNQUFNLDBCQUEwQixDQUFDO0FBRTdEOztHQUVHO0FBQ0gsTUFBTSxPQUFPLGVBQWU7SUFDMUI7OztPQUdHO0lBQ0gsWUFBb0IsTUFBYztRQUFkLFdBQU0sR0FBTixNQUFNLENBQVE7SUFBRyxDQUFDO0lBRXRDOztPQUVHO0lBQ0gsWUFBWTtRQUNWLE1BQU0sT0FBTyxHQUFHLElBQUksY0FBYyxFQUFFLENBQUM7UUFDckMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsRUFBRSxFQUFFLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxDQUFDLENBQUM7UUFDN0MsT0FBTyxPQUFPLENBQUMsT0FBTyxDQUFDLElBQUksaUJBQWlCLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxDQUFDLENBQUM7SUFDL0QsQ0FBQztDQUNGIn0=