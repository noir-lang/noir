import { MessagePort, TransferListItem } from 'worker_threads';
import { Socket } from '../interface/socket.js';

/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class NodeListenerSocket implements Socket {
  constructor(private port: MessagePort) {}

  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.port.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  registerHandler(cb: (msg: any) => any): void {
    this.port.on('message', cb);
  }

  close() {
    void this.send(undefined);
    this.port.removeAllListeners();
    this.port.close();
  }
}
