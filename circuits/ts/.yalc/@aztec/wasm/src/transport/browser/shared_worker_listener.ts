import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * See https://developer.mozilla.org/en-US/docs/Web/API/SharedWorkerGlobalScope.
 */
declare interface SharedWorkerGlobalScope {
  /**
   * Fired on shared workers when a new client connects.
   */
  onconnect: any;
}

/**
 * Listens for connections to a shared worker.
 */
export class SharedWorkerListener extends EventEmitter implements Listener {
  /**
   *
   * @param worker
   */
  constructor(private worker: SharedWorkerGlobalScope) {
    super();
  }

  /**
   *
   */
  open() {
    this.worker.onconnect = this.handleMessageEvent;
  }

  /**
   *
   */
  close() {
    this.worker.onconnect = () => {};
  }

  /**
   *
   * @param event
   */
  private handleMessageEvent = (event: MessageEvent) => {
    const [port] = event.ports;
    if (!port) {
      return;
    }
    this.emit('new_socket', new MessagePortSocket(port));
  };
}
