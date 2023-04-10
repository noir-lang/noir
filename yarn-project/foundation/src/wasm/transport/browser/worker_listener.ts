import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
import { MessagePortSocket } from './message_port_socket.js';

/**
 * Represents the global scope of a dedicated worker.
 * Provides functionality for handling communication between the main thread and the worker, including receiving messages and managing the worker's lifecycle.
 */
declare interface DedicatedWorkerGlobalScope {
  /**
   * The event handler for incoming messages from the main thread to the dedicated worker.
   */
  onmessage: any;
}

/**
 * The WorkerListener class is responsible for managing communication between the main thread and a DedicatedWorkerGlobalScope.
 * It extends EventEmitter and implements the Listener interface, emitting 'new_socket' events when a new MessagePortSocket is created.
 * The open() method activates the listener by setting a handler for incoming messages, while the close() method deactivates it.
 */
export class WorkerListener extends EventEmitter implements Listener {
  constructor(private worker: DedicatedWorkerGlobalScope) {
    super();
  }

  /**
   * Opens the worker listener to start receiving messages from the DedicatedWorkerGlobalScope.
   * Upon opening, it sets the 'onmessage' event handler of the worker to 'handleMessageEvent'.
   * This method should be called when you want to start listening for new MessagePort connections from the worker.
   */
  open() {
    this.worker.onmessage = this.handleMessageEvent;
  }

  /**
   * Closes the WorkerListener by removing the event listener for 'onmessage' from the worker.
   * This function is useful when you want to stop listening for new connections from the worker.
   */
  close() {
    this.worker.onmessage = () => {};
  }

  private handleMessageEvent = (event: MessageEvent) => {
    const [port] = event.ports;
    if (!port) {
      return;
    }
    this.emit('new_socket', new MessagePortSocket(port));
  };
}
