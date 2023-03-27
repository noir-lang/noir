import { parentPort } from 'worker_threads';
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { NodeListenerSocket } from './node_listener_socket.js';

export class NodeListener extends EventEmitter implements Listener {
  constructor() {
    super();
  }

  open() {
    this.emit('new_socket', new NodeListenerSocket(parentPort as any));
  }

  close() {}
}
