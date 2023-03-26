import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';
/**
 * Connector implementation which wraps a SharedWorker.
 */
export declare class SharedWorkerConnector implements Connector {
    private worker;
    /**
     * Create a SharedWorkerConnector.
     * @param worker - A shared worker.
     */
    constructor(worker: SharedWorker);
    /**
     * Create a Socket implementation with our mesage port.
     * @returns The socket.
     */
    createSocket(): Promise<MessagePortSocket>;
}
//# sourceMappingURL=shared_worker_connector.d.ts.map