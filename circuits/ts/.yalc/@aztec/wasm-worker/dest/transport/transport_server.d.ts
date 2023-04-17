import { Listener } from './interface/listener.js';
/**
 * Keeps track of clients, providing a broadcast, and request/response api with multiplexing.
 */
export declare class TransportServer<Payload> {
    private listener;
    private msgHandlerFn;
    private sockets;
    constructor(listener: Listener, msgHandlerFn: (msg: Payload) => Promise<any>);
    /**
     * Start accepting new connections.
     */
    start(): void;
    /**
     * Stops accepting new connections. It doesn't close existing sockets.
     * It's expected the clients will gracefully complete by closing their end, sending an `undefined` message.
     */
    stop(): void;
    /**
     * Broadcast a message.
     * @param msg - The message.
     */
    broadcast(msg: Payload): Promise<void>;
    /**
     * New socket registration.
     * @param socket - The socket to register.
     */
    private handleNewSocket;
    /**
     * Detect the 'transferables' argument to our socket from our message
     * handler return type.
     * @param data - The return object.
     * @returns - The data and the.
     */
    private getPayloadAndTransfers;
    /**
     * Handles a socket message from a listener.
     * @param socket - The socket.
     * @param requestMessage - The message to handle.
     * @returns The socket response.
     */
    private handleSocketMessage;
}
//# sourceMappingURL=transport_server.d.ts.map