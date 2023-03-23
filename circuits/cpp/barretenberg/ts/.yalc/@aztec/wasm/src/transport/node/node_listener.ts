import { parentPort } from 'worker_threads';
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { NodeListenerSocket } from './node_listener_socket.js';

/**
 * A socket listener that works with Node.
 */
export class NodeListener extends EventEmitter implements Listener {
  constructor() {
    super();
  }

  /**
   * Open the listener.
   */
  open() {
    this.emit('new_socket', new NodeListenerSocket(parentPort as any));
  }

  /**
   * Close the listener.
   */
  close() {}
}
