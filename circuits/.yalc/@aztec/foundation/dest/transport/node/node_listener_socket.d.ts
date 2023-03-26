/// <reference types="node" resolution-mode="require"/>
import { MessagePort } from 'worker_threads';
import { Socket } from '../interface/socket.js';
/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export declare class NodeListenerSocket implements Socket {
    private port;
    constructor(port: MessagePort);
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    registerHandler(cb: (msg: any) => any): void;
    close(): void;
}
//# sourceMappingURL=node_listener_socket.d.ts.map