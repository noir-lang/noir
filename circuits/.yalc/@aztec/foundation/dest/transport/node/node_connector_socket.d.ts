/// <reference types="node" resolution-mode="require"/>
import { Worker } from 'worker_threads';
import { Socket } from '../interface/socket.js';
export declare class NodeConnectorSocket implements Socket {
    private worker;
    constructor(worker: Worker);
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    registerHandler(cb: (msg: any) => any): void;
    close(): void;
}
//# sourceMappingURL=node_connector_socket.d.ts.map