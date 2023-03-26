import { TransferListItem, Worker } from 'worker_threads';
import { Socket } from '../interface/socket.js';

export class NodeConnectorSocket implements Socket {
  constructor(private worker: Worker) {}

  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.worker.postMessage(msg, transfer as TransferListItem[]);
    return Promise.resolve();
  }

  registerHandler(cb: (msg: any) => any): void {
    this.worker.on('message', cb);
  }

  close() {
    void this.send(undefined);
    this.worker.removeAllListeners();
  }
}
