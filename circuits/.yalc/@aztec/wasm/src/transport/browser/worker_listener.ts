import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 *
 */
declare interface DedicatedWorkerGlobalScope {
  /**
   *
   */
  onmessage: any;
}

/**
 *
 */
export class WorkerListener extends EventEmitter implements Listener {
  /**
   *
   * @param worker
   */
  constructor(private worker: DedicatedWorkerGlobalScope) {
    super();
  }

  /**
   *
   */
  open() {
    this.worker.onmessage = this.handleMessageEvent;
  }

  /**
   *
   */
  close() {
    this.worker.onmessage = () => {};
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
