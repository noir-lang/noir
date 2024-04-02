import { type TransferListItem, type Worker } from 'worker_threads';

import { type Socket } from '../interface/socket.js';

/**
 * NodeConnectorSocket is a wrapper class that implements the Socket interface for messaging between
 * the main thread and worker threads in a Node.js environment. It uses the Worker API for
 * communication by sending and receiving messages through postMessage and handling messages using
 * event listeners.
 *
 * The send method sends messages to the worker thread, and the registerHandler method registers a
 * callback function to handle incoming messages from the worker. The close method cleans up
 * resources when the socket is no longer needed.
 */
export class NodeConnectorSocket implements Socket {
  constructor(private worker: Worker) {}

  /**
   * Sends a message from the NodeConnectorSocket instance to the associated worker thread.
   * The 'msg' can be any data type and 'transfer' is an optional array of transferable objects
   * that can be transferred with zero-copy semantics. The function returns a resolved Promise
   * once the message has been posted.
   *
   * @param msg - The message to send to the worker thread.
   * @param transfer - Optional array of Transferable objects to transfer ownership alongside the message.
   * @returns A Promise that resolves when the message has been posted.
   */
  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.worker.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  /**
   * Registers a callback function to handle incoming messages from the worker.
   * The provided callback will be executed whenever a message is received from
   * the worker, passing the message as its single argument.
   *
   * @param cb - The callback function to be called when a message is received.
   */
  registerHandler(cb: (msg: any) => any): void {
    this.worker.on('message', cb);
  }

  /**
   * Closes the worker connection and removes all event listeners.
   * Sends an undefined message to the worker for graceful termination.
   */
  close() {
    void this.send(undefined);
    this.worker.removeAllListeners();
  }
}
