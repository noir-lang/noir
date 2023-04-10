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
  constructor(private worker: SharedWorkerGlobalScope) {
    super();
  }

  /**
   * Opens the shared worker and starts listening for incoming connections.
   * The 'onconnect' event of the SharedWorkerGlobalScope is set to handle incoming connection events,
   * creating a new socket for each connection.
   */
  open() {
    this.worker.onconnect = this.handleMessageEvent;
  }

  /**
   * Closes the listener and stops handling new connections to the shared worker.
   * This function removes the event handler for the 'onconnect' event, effectively
   * preventing the SharedWorkerListener from emitting any further 'new_socket' events.
   */
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
