import { type Worker } from 'worker_threads';

import { type Connector } from '../interface/connector.js';
import { NodeConnectorSocket } from './node_connector_socket.js';

/**
 * The NodeConnector class is a concrete implementation of the Connector interface, utilizing worker_threads for
 * efficient parallel execution. This class provides an easy way to establish a connection with a Worker instance,
 * allowing seamless communication via sockets.
 *
 * @example
 * const worker = new Worker('./path/to/worker.js');
 * const nodeConnector = new NodeConnector(worker);
 * const socket = await nodeConnector.createSocket();
 * socket.send('Hello from main thread!');
 */
export class NodeConnector implements Connector {
  constructor(private worker: Worker) {}

  /**
   * Creates a new instance of NodeConnectorSocket using the worker provided in the constructor.
   * The createSocket method is used to establish connections using the worker_threads module,
   * allowing for efficient and fast communication between different parts of the application.
   *
   * @returns A Promise that resolves to a newly created NodeConnectorSocket instance.
   */
  createSocket() {
    return Promise.resolve(new NodeConnectorSocket(this.worker));
  }
}
