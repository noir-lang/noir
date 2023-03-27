import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';

export class SharedWorkerConnector implements Connector {
  constructor(private worker: SharedWorker) {}

  createSocket() {
    return Promise.resolve(new MessagePortSocket(this.worker.port));
  }
}
