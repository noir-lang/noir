import { Socket } from '../interface/socket.js';
/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export declare class MessagePortSocket implements Socket {
    private port;
    /**
     * Create a MessagePortSocket.
     * @param port - MessagePort object to wrap.
     */
    constructor(port: MessagePort);
    /**
     * Send a message over our message port.
     * @param msg - The message.
     * @param transfer - Objects to transfer ownership of.
     */
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    /**
     * Add a message handler.
     * @param cb - The handler.
     */
    registerHandler(cb: (msg: any) => any): void;
    /**
     * Close this message port.
     */
    close(): void;
}
//# sourceMappingURL=message_port_socket.d.ts.map