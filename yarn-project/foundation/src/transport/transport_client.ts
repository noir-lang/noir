import EventEmitter from 'events';
import { format } from 'util';

import { createDebugLogger } from '../log/index.js';
import { type EventMessage, type ResponseMessage, isEventMessage } from './dispatch/messages.js';
import { type Connector } from './interface/connector.js';
import { type Socket } from './interface/socket.js';

const log = createDebugLogger('aztec:transport_client');

/**
 * Represents a pending request in the TransportClient.
 * Contains information about the message ID, and resolve/reject functions for handling responses.
 * Used to track and manage asynchronous request/response communication with the TransportServer.
 */
interface PendingRequest {
  /**
   * The unique message identifier used for tracking and matching request/response pairs.
   */
  msgId: number;
  // eslint-disable-next-line jsdoc/require-jsdoc
  resolve(data: any): void;
  // eslint-disable-next-line jsdoc/require-jsdoc
  reject(error: Error): void;
}

/**
 * Represents a transport client for communication between TransportServer and clients.
 * Provides request/response functionality, event handling, and multiplexing support
 * for efficient and concurrent communication with a corresponding TransportServer.
 */
export interface ITransportClient<Payload> extends EventEmitter {
  // eslint-disable-next-line jsdoc/require-jsdoc
  on(name: 'event_msg', handler: (payload: Payload) => void): this;
  // eslint-disable-next-line jsdoc/require-jsdoc
  emit(name: 'event_msg', payload: Payload): boolean;
}

/**
 * A TransportClient provides a request/response and event api to a corresponding TransportServer.
 * If `broadcast` is called on TransportServer, TransportClients will emit an `event_msg`.
 * The `request` method will block until a response is returned from the TransportServer's dispatch function.
 * Request multiplexing is supported.
 */
export class TransportClient<Payload> extends EventEmitter {
  private msgId = 0;
  private pendingRequests: PendingRequest[] = [];
  private socket?: Socket;

  constructor(private transportConnect: Connector) {
    super();
  }

  /**
   * Initializes and opens the socket connection for the TransportClient.
   * This method creates a new Socket instance using the provided Connector,
   * registers a handler for incoming messages, and establishes the connection.
   * It should be called before making any requests or handling events.
   *
   * @throws An error if the socket is already open or there's an issue opening the connection.
   * @returns A Promise that resolves when the socket connection is successfully opened.
   */
  async open() {
    this.socket = await this.transportConnect.createSocket();
    this.socket.registerHandler(msg => this.handleSocketMessage(msg));
  }

  /**
   * Close the transport client's socket connection and remove all event listeners.
   * This method should be called when the client is no longer needed to ensure proper cleanup
   * and prevent potential memory leaks. Once closed, the client cannot be reused and a new
   * instance must be created if another connection is needed.
   */
  close() {
    this.socket?.close();
    this.socket = undefined;
    this.removeAllListeners();
  }

  /**
   * Sends a request to the TransportServer with the given payload and transferable objects.
   * The method will block until a response from the TransportServer's dispatch function is returned.
   * Request multiplexing is supported, allowing multiple requests to be sent concurrently.
   *
   * @param payload - The message payload to send to the server.
   * @param transfer - An optional array of ArrayBuffer, MessagePort, or ImageBitmap objects to transfer ownership.
   * @returns A Promise that resolves with the server's response data or rejects with an error message.
   */
  request(payload: Payload, transfer?: Transferable[]) {
    if (!this.socket) {
      throw new Error('Socket not open.');
    }
    const msgId = this.msgId++;
    const msg = { msgId, payload };
    log.debug(format(`->`, msg));
    return new Promise<any>((resolve, reject) => {
      this.pendingRequests.push({ resolve, reject, msgId });
      this.socket!.send(msg, transfer).catch(reject);
    });
  }

  /**
   * Handles incoming socket messages from the TransportServer, such as ResponseMessage and EventMessage.
   * If it's an EventMessage, emits an 'event_msg' event with the payload.
   * If it's a ResponseMessage, resolves or rejects the corresponding pending request based on the message content.
   *
   * @param msg - The ResponseMessage or EventMessage received from the TransportServer, or undefined if the remote socket closed.
   */
  private handleSocketMessage(msg: ResponseMessage<Payload> | EventMessage<Payload> | undefined) {
    if (msg === undefined) {
      // The remote socket closed.
      this.close();
      return;
    }
    log.debug(format(`<-`, msg));
    if (isEventMessage(msg)) {
      this.emit('event_msg', msg.payload);
      return;
    }
    const reqIndex = this.pendingRequests.findIndex(r => r.msgId === msg.msgId);
    if (reqIndex === -1) {
      return;
    }
    const [pending] = this.pendingRequests.splice(reqIndex, 1);
    if (msg.error) {
      pending.reject(new Error(msg.error));
    } else {
      pending.resolve(msg.payload);
    }
  }
}
