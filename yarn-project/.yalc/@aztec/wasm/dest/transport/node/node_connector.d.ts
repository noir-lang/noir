/// <reference types="node" resolution-mode="require"/>
import { Worker } from 'worker_threads';
import { Connector } from '../interface/connector.js';
import { NodeConnectorSocket } from './node_connector_socket.js';
/**
 * Creates sockets backed by a Node worker.
 */
export declare class NodeConnector implements Connector {
    private worker;
    constructor(worker: Worker);
    /**
     * Creates a socket backed by a node worker.
     * @returns The socket.
     */
    createSocket(): Promise<NodeConnectorSocket>;
}
//# sourceMappingURL=node_connector.d.ts.map