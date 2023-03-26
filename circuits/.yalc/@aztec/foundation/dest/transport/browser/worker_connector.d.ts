import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';
export declare class WorkerConnector implements Connector {
    private worker;
    constructor(worker: Worker);
    createSocket(): Promise<MessagePortSocket>;
}
//# sourceMappingURL=worker_connector.d.ts.map