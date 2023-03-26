import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 *
 */
export class WorkerConnector implements Connector {
  /**
   *
   * @param worker
   */
  constructor(private worker: Worker) {}

  /**
   *
   */
  createSocket() {
    const channel = new MessageChannel();
    this.worker.postMessage('', [channel.port2]);
    return Promise.resolve(new MessagePortSocket(channel.port1));
  }
}
