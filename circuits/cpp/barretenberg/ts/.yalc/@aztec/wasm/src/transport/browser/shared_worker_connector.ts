import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * Connector implementation which wraps a SharedWorker.
 */
export class SharedWorkerConnector implements Connector {
  /**
   * Create a SharedWorkerConnector.
   * @param worker - A shared worker.
   */
  constructor(private worker: SharedWorker) {}

  /**
   * Create a Socket implementation with our mesage port.
   * @returns The socket.
   */
  createSocket() {
    return Promise.resolve(new MessagePortSocket(this.worker.port));
  }
}
