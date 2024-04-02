import EventEmitter from 'events';

import { type Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * Represents a DedicatedWorkerGlobalScope, which is the global execution context for a dedicated worker.
 * Provides properties and methods to manage the worker's lifecycle and communication with other threads or workers.
 */
declare interface DedicatedWorkerGlobalScope {
  /**
   * Handler for incoming messages from other threads or workers.
   */
  onmessage: any;
}

/**
 * WorkerListener is a class that extends EventEmitter and implements the Listener interface.
 * It listens for incoming connections on a dedicated worker global scope, and emits a 'new_socket' event
 * with a MessagePortSocket instance for each new connection. This allows applications to communicate
 * with other workers or main thread through the MessagePortSocket abstraction.
 *
 * The open() method starts listening for incoming connections, while the close() method stops it.
 */
export class WorkerListener extends EventEmitter implements Listener {
  constructor(private worker: DedicatedWorkerGlobalScope) {
    super();
  }

  /**
   * Initializes the WorkerListener by setting the 'onmessage' event handler of the worker.
   * The 'onmessage' event will be triggered when the worker receives a message, and it will then
   * call the handleMessageEvent method to handle incoming connections.
   */
  open() {
    this.worker.onmessage = this.handleMessageEvent;
  }

  /**
   * Close the worker listener by removing the 'onmessage' event handler.
   * This method effectively stops the WorkerListener from reacting to new incoming messages.
   */
  close() {
    this.worker.onmessage = () => {};
  }

  private handleMessageEvent = (event: MessageEvent) => {
    const [port] = event.ports;
    if (!port) {
      return;
    }
    this.emit('new_socket', new MessagePortSocket(port));
  };
}
