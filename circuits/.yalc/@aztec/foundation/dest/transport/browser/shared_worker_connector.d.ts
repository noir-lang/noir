import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';
export declare class SharedWorkerConnector implements Connector {
    private worker;
    constructor(worker: SharedWorker);
    createSocket(): Promise<MessagePortSocket>;
}
//# sourceMappingURL=shared_worker_connector.d.ts.map