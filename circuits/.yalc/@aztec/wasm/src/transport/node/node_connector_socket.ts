import { TransferListItem, Worker } from 'worker_threads';
import { Socket } from '../interface/socket.js';

/**
 * A socket implementation using a Node worker.
 */
export class NodeConnectorSocket implements Socket {
  constructor(private worker: Worker) {}

  /**
   * Send a message.
   * @param msg - The message.
   * @param transfer - Objects to transfer ownership of.
   * @returns A void promise.
   */
  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.worker.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  /**
   * Register a message handler.
   * @param cb - The handler function.
   */
  registerHandler(cb: (msg: any) => any): void {
    this.worker.on('message', cb);
  }

  /**
   * Remove all listeners from our worker.
   */
  close() {
    void this.send(undefined);
    this.worker.removeAllListeners();
  }
}
