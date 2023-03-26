import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

declare interface SharedWorkerGlobalScope {
  onconnect: (...args: any) => any;
}

export class SharedWorkerListener extends EventEmitter implements Listener {
  constructor(private worker: SharedWorkerGlobalScope) {
    super();
  }

  open() {
    this.worker.onconnect = this.handleMessageEvent;
  }

  close() {
    this.worker.onconnect = () => {};
  }

  private handleMessageEvent = (event: MessageEvent) => {
    const [port] = event.ports;
    if (!port) {
      return;
    }
    this.emit('new_socket', new MessagePortSocket(port));
  };
}
