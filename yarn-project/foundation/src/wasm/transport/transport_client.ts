import EventEmitter from 'events';
import { createDebugLogger } from '../../log/index.js';
import { EventMessage, isEventMessage, ResponseMessage } from './dispatch/messages.js';
import { Connector } from './interface/connector.js';
import { Socket } from './interface/socket.js';

const debug = createDebugLogger('aztec:transport_client');

/**
 * A pending request.
 */
interface PendingRequest {
  /**
   * The message ID.
   */
  msgId: number;
  resolve(data: any): void;
  reject(error: Error): void;
}

/**
 * Augments the TransportClient class with more precise EventEmitter types.
 */
export interface TransportClient<Payload> extends EventEmitter {
  on(name: 'event_msg', handler: (payload: Payload) => void): this;
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
   * Create and register our socket using our Connector.
   */
  async open() {
    this.socket = await this.transportConnect.createSocket();
    this.socket.registerHandler(msg => this.handleSocketMessage(msg));
  }

  /**
   * Close this and stop listening for messages.
   */
  close() {
    this.socket?.close();
    this.socket = undefined;
    this.removeAllListeners();
  }

  /**
   * Queue a request.
   * @param payload - The request payload.
   * @param transfer - Objects to transfer ownership of.
   * @returns A promise of the query result.
   */
  request(payload: Payload, transfer?: Transferable[]) {
    if (!this.socket) {
      throw new Error('Socket not open.');
    }
    const msgId = this.msgId++;
    const msg = { msgId, payload };
    debug(`->`, msg);
    return new Promise<any>((resolve, reject) => {
      this.pendingRequests.push({ resolve, reject, msgId });
      this.socket!.send(msg, transfer).catch(reject);
    });
  }

  /**
   * Handle an incoming socket message.
   * @param msg - The message.
   */
  private handleSocketMessage(msg: ResponseMessage<Payload> | EventMessage<Payload> | undefined) {
    if (msg === undefined) {
      // The remote socket closed.
      this.close();
      return;
    }
    debug(`<-`, msg);
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
