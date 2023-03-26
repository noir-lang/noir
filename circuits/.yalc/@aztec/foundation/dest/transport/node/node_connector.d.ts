/// <reference types="node" resolution-mode="require"/>
import { Worker } from 'worker_threads';
import { Connector } from '../interface/connector.js';
import { NodeConnectorSocket } from './node_connector_socket.js';
export declare class NodeConnector implements Connector {
    private worker;
    constructor(worker: Worker);
    createSocket(): Promise<NodeConnectorSocket>;
}
//# sourceMappingURL=node_connector.d.ts.map