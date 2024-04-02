import EventEmitter from 'events';
import { parentPort } from 'worker_threads';

import { type Listener } from '../interface/listener.js';
import { NodeListenerSocket } from './node_listener_socket.js';

/**
 * NodeListener is an event-driven class that extends EventEmitter and implements the Listener interface.
 * It provides methods to open and close communication with a worker thread using the NodeListenerSocket.
 * The 'new_socket' event is emitted when a new NodeListenerSocket instance is created, allowing for
 * efficient processing of incoming messages from the parent thread.
 */
export class NodeListener extends EventEmitter implements Listener {
  constructor() {
    super();
  }

  /**
   * Opens a new connection to a parent worker thread and emits an event with the created NodeListenerSocket instance.
   * The 'new_socket' event can be listened for, providing access to the newly created NodeListenerSocket.
   *
   * Fires NodeListener#new_socket.
   */
  open() {
    this.emit('new_socket', new NodeListenerSocket(parentPort as any));
  }

  /**
   * Closes the NodeListener instance.
   * This method currently has no implementation, as there is no need to perform any actions
   * when closing a NodeListener. It exists for compatibility with the Listener interface.
   */
  close() {}
}
