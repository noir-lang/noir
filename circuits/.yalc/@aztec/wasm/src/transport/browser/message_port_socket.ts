import { Socket } from '../interface/socket.js';

/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class MessagePortSocket implements Socket {
  /**
   * Create a MessagePortSocket.
   * @param port - MessagePort object to wrap.
   */
  constructor(private port: MessagePort) {}

  /**
   * Send a message over our message port.
   * @param msg - The message.
   * @param transfer - Objects to transfer ownership of.
   */
  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.port.postMessage(msg, transfer);
    return Promise.resolve();
  }

  /**
   * Add a message handler.
   * @param cb - The handler.
   */
  registerHandler(cb: (msg: any) => any): void {
    this.port.onmessage = event => cb(event.data);
  }

  /**
   * Close this message port.
   */
  close() {
    void this.send(undefined);
    this.port.onmessage = null;
    this.port.close();
  }
}
