import { type MessagePort, type TransferListItem } from 'worker_threads';

import { type Socket } from '../interface/socket.js';

/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class NodeListenerSocket implements Socket {
  constructor(private port: MessagePort) {}

  /**
   * Sends a message through the MessagePort along with any provided Transferables.
   * The transfer list allows for efficient sending of certain types of data,
   * such as ArrayBuffer, ImageBitmap, and MessagePort.
   * The Promise resolves once the message has been successfully sent.
   *
   * @param msg - The message to be sent through the MessagePort.
   * @param transfer - An optional array of Transferable objects to be transferred.
   * @returns A Promise that resolves once the message has been sent.
   */
  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.port.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  /**
   * Registers a callback function to handle incoming messages from the MessagePort.
   * When a message is received, the provided callback function will be invoked with
   * the received message as its argument. This method allows for efficient and
   * dynamic handling of incoming data in a NodeListenerSocket instance.
   *
   * @param cb - The callback function to process incoming messages.
   */
  registerHandler(cb: (msg: any) => any): void {
    this.port.on('message', cb);
  }

  /**
   * Closes the NodeListenerSocket instance, removing all listeners and closing the underlying MessagePort.
   * Sends an undefined message to notify any connected ports about the closure before removing event listeners
   * and cleaning up resources. This method should be called when the socket is no longer needed to avoid memory leaks.
   */
  close() {
    void this.send(undefined);
    this.port.removeAllListeners();
    this.port.close();
  }
}
