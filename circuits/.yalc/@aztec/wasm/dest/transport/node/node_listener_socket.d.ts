/// <reference types="node" resolution-mode="require"/>
import { MessagePort } from 'worker_threads';
import { Socket } from '../interface/socket.js';
/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export declare class NodeListenerSocket implements Socket {
    private port;
    constructor(port: MessagePort);
    /**
     * Send a message over this port.
     * @param msg - The message.
     * @param transfer - Transferable objects.
     * @returns A void promise.
     */
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    /**
     * Add a handler to this port.
     * @param cb - The handler function.
     */
    registerHandler(cb: (msg: any) => any): void;
    /**
     * Close this socket.
     */
    close(): void;
}
//# sourceMappingURL=node_listener_socket.d.ts.map