import { Connector } from '../interface/connector.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * The WorkerConnector class is a concrete implementation of the Connector interface,
 * responsible for creating sockets wrapped with MessagePort instances.
 * These sockets communicate with Web Workers to establish a connection, allowing
 * communication between the main thread and worker threads in a concurrent, non-blocking manner.
 * The WorkerConnector ensures that messages are dispatched to the appropriate worker thread,
 * making it an essential component in handling background tasks and improving performance.
 */
export class WorkerConnector implements Connector {
  constructor(private worker: Worker) {}

  /**
   * Creates a new MessagePortSocket instance by establishing a communication channel with the Worker.
   * The function sets up a MessageChannel, posts a message to the Worker with one of the ports,
   * and resolves to a new MessagePortSocket with the other port.
   *
   * @returns A Promise that resolves to a MessagePortSocket instance for bi-directional communication with the Worker.
   */
  createSocket() {
    const channel = new MessageChannel();
    this.worker.postMessage('', [channel.port2]);
    return Promise.resolve(new MessagePortSocket(channel.port1));
  }
}
