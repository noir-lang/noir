/**
 * Represents one end of a socket connection.
 * A message sent via `send` will be handled by the corresponding Socket's handler function at the other end.
 * Implementations could use e.g. MessagePorts for communication between browser workers,
 * or WebSockets for communication between processes.
 * If `registerHandler` callback receives `undefined` that signals the other end closed.
 */
export interface Socket {
  // eslint-disable-next-line jsdoc/require-jsdoc
  send(msg: any, transfer?: Transferable[]): Promise<void>;
  // eslint-disable-next-line jsdoc/require-jsdoc
  registerHandler(cb: (msg: any) => any): void;
  // eslint-disable-next-line jsdoc/require-jsdoc
  close(): void;
}
