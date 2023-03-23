import { MessagePort, TransferListItem } from 'worker_threads';
import { Socket } from '../interface/socket.js';

/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class NodeListenerSocket implements Socket {
  constructor(private port: MessagePort) {}

  /**
   * Send a message over this port.
   * @param msg - The message.
   * @param transfer - Transferable objects.
   * @returns A void promise.
   */
  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.port.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  /**
   * Add a handler to this port.
   * @param cb - The handler function.
   */
  registerHandler(cb: (msg: any) => any): void {
    this.port.on('message', cb);
  }

  /**
   * Close this socket.
   */
  close() {
    void this.send(undefined);
    this.port.removeAllListeners();
    this.port.close();
  }
}
