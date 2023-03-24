/**
 * Represents one end of a socket connection.
 * A message sent via `send` will be handled by the corresponding Socket's handler function at the other end.
 * Implementations could use e.g. MessagePorts for communication between browser workers,
 * or WebSockets for communication between processes.
 * If `registerHandler` callback receives `undefined` that signals the other end closed.
 */
export interface Socket {
    send(msg: any, transfer?: Transferable[]): Promise<void>;
    registerHandler(cb: (msg: any) => any): void;
    close(): void;
}
//# sourceMappingURL=socket.d.ts.map