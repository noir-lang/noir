import { Listener } from './interface/listener.js';
/**
 * Keeps track of clients, providing a broadcast, and request/response api with multiplexing.
 */
export declare class TransportServer<Payload> {
    private listener;
    private msgHandlerFn;
    private sockets;
    constructor(listener: Listener, msgHandlerFn: (msg: Payload) => Promise<any>);
    start(): void;
    /**
     * Stops accepting new connections. It doesn't close existing sockets.
     * It's expected the clients will gracefully complete by closing their end, sending an `undefined` message.
     */
    stop(): void;
    broadcast(msg: Payload): Promise<void>;
    private handleNewSocket;
    /**
     * Detect the 'transferables' argument to our socket from our message
     * handler return type.
     */
    private getPayloadAndTransfers;
    private handleSocketMessage;
}
//# sourceMappingURL=transport_server.d.ts.map