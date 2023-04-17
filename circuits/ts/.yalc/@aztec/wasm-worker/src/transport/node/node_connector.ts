import { Worker } from 'worker_threads';
import { Connector } from '../interface/connector.js';
import { NodeConnectorSocket } from './node_connector_socket.js';

/**
 * Creates sockets backed by a Node worker.
 */
export class NodeConnector implements Connector {
  constructor(private worker: Worker) {}

  /**
   * Creates a socket backed by a node worker.
   * @returns The socket.
   */
  createSocket() {
    return Promise.resolve(new NodeConnectorSocket(this.worker));
  }
}
