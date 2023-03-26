import { Socket } from '../interface/socket.js';

/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class MessagePortSocket implements Socket {
  constructor(private port: MessagePort) {}

  send(msg: any, transfer: Transferable[] = []): Promise<void> {
    this.port.postMessage(msg, transfer);
    return Promise.resolve();
  }

  registerHandler(cb: (msg: any) => any): void {
    this.port.onmessage = event => cb(event.data);
  }

  close() {
    void this.send(undefined);
    this.port.onmessage = null;
    this.port.close();
  }
}
