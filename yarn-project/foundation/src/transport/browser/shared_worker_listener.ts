import EventEmitter from 'events';

import { type Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * Represents the global scope of a Shared Worker.
 * Provides functionality to handle incoming connections and manage communication with other scripts
 * running in a shared context, enabling concurrent access and efficient resource sharing among those scripts.
 */
declare interface SharedWorkerGlobalScope {
  /**
   * Event handler for new connections to the Shared Worker.
   */
  onconnect: (...args: any) => any;
}

/**
 * SharedWorkerListener is an extension of the EventEmitter class that implements the Listener interface.
 * It provides functionality to handle incoming messages from a shared worker and emit events for new sockets
 * created in response to these incoming connections. This class is meant to be used in the context of managing
 * MessagePort connections within the SharedWorkerGlobalScope.
 */
export class SharedWorkerListener extends EventEmitter implements Listener {
  constructor(private worker: SharedWorkerGlobalScope) {
    super();
  }

  /**
   * Initializes the shared worker listener by assigning the 'handleMessageEvent' method as the event handler
   * for the 'onconnect' event of the SharedWorkerGlobalScope. The 'handleMessageEvent' function will be called
   * whenever a new connection is established with the shared worker.
   */
  open() {
    this.worker.onconnect = this.handleMessageEvent;
  }

  /**
   * Closes the SharedWorkerListener by detaching the 'onconnect' event handler.
   * This stops the listener from emitting new sockets on incoming connections.
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
