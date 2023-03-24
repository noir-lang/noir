/// <reference types="node" resolution-mode="require"/>
import { Worker } from 'worker_threads';
import { Socket } from '../interface/socket.js';
/**
 * A socket implementation using a Node worker.
 */
export declare class NodeConnectorSocket implements Socket {
    private worker;
    constructor(worker: Worker);
    /**
     * Send a message.
     * @param msg - The message.
     * @param transfer - Objects to transfer ownership of.
     * @returns A void promise.
     */
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    /**
     * Register a message handler.
     * @param cb - The handler function.
     */
    registerHandler(cb: (msg: any) => any): void;
    /**
     * Remove all listeners from our worker.
     */
    close(): void;
}
//# sourceMappingURL=node_connector_socket.d.ts.map