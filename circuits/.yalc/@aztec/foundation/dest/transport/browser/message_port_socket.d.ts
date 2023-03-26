import { Socket } from '../interface/socket.js';
/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export declare class MessagePortSocket implements Socket {
    private port;
    constructor(port: MessagePort);
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    registerHandler(cb: (msg: any) => any): void;
    close(): void;
}
//# sourceMappingURL=message_port_socket.d.ts.map