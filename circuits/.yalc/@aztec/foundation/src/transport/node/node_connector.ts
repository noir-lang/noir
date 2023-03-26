import { Worker } from 'worker_threads';
import { Connector } from '../interface/connector.js';
import { NodeConnectorSocket } from './node_connector_socket.js';

export class NodeConnector implements Connector {
  constructor(private worker: Worker) {}

  createSocket() {
    return Promise.resolve(new NodeConnectorSocket(this.worker));
  }
}
