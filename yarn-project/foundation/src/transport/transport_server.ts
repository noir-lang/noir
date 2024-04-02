import { type RequestMessage, type ResponseMessage } from './dispatch/messages.js';
import { type Listener } from './interface/listener.js';
import { type Socket } from './interface/socket.js';
import { isTransferDescriptor } from './interface/transferable.js';

/**
 * Keeps track of clients, providing a broadcast, and request/response api with multiplexing.
 */
export class TransportServer<Payload> {
  private sockets: Socket[] = [];

  constructor(private listener: Listener, private msgHandlerFn: (msg: Payload) => Promise<any>) {}

  /**
   * Starts the TransportServer, allowing it to accept new connections and handle incoming messages.
   * The server will listen for 'new_socket' events from the underlying listener and invoke the provided message handler function
   * for each received message. The server remains active until the 'stop' method is called.
   */
  start() {
    this.listener.on('new_socket', client => this.handleNewSocket(client));
    this.listener.open();
  }

  /**
   * Stops accepting new connections. It doesn't close existing sockets.
   * It's expected the clients will gracefully complete by closing their end, sending an `undefined` message.
   */
  stop() {
    this.listener.close();
  }

  /**
   * Sends a broadcast message to all connected clients.
   * The given payload will be sent to all the clients currently connected to the TransportServer.
   * It waits for all the messages to be sent and resolves when they are all sent successfully.
   *
   * @param msg - The payload to broadcast to all connected clients.
   * @returns A Promise that resolves when all messages have been sent successfully.
   */
  async broadcast(msg: Payload) {
    await Promise.all(this.sockets.map(s => s.send({ payload: msg })));
  }

  /**
   * Handles the addition of a new socket to the server by registering a message handler for the client
   * and adding the socket to the list of active sockets. The message handler processes incoming messages
   * from the client, including detecting client disconnection and removing the closed socket.
   *
   * @param socket - The new Socket instance that has connected to the server.
   */
  private handleNewSocket(socket: Socket) {
    socket.registerHandler(async msg => {
      if (msg === undefined) {
        // Client socket has closed. Remove it from the list of sockets. Call close on it for any cleanup.
        const socketIndex = this.sockets.findIndex(s => s === socket);
        const [closingSocket] = this.sockets.splice(socketIndex, 1);
        closingSocket.close();
        return;
      }
      return await this.handleSocketMessage(socket, msg);
    });
    this.sockets.push(socket);
  }

  /**
   * Detect the 'transferables' argument to our socket from our message
   * handler return type.
   * @param data - The compound payload data.
   * @returns The split data and transferables.
   */
  private getPayloadAndTransfers(data: any): [any, Transferable[]] {
    if (isTransferDescriptor(data)) {
      // We treat PayloadWithTransfers specially so that we're able to
      // attach transferables while keeping a simple return-type based usage
      return [data.send, data.transferables];
    }
    if (data instanceof Uint8Array) {
      // We may want to devise a better solution to this. We maybe given a view over a non cloneable/transferrable
      // ArrayBuffer (such as a view over wasm memory). In this case we want to take a copy, and then transfer it.
      const respPayload = data instanceof Uint8Array && ArrayBuffer.isView(data) ? new Uint8Array(data) : data;
      const transferables = data instanceof Uint8Array ? [respPayload.buffer] : [];
      return [respPayload, transferables];
    }
    return [data, []];
  }
  /**
   * Handles incoming socket messages, processing the request and sending back a response.
   * This function is responsible for invoking the registered message handler function with the received
   * payload, extracting the result and transferables, and sending a response message back to the client.
   * In case of an error during message handling, it sends an error response with the stack trace.
   *
   * @param socket - The Socket instance from which the message was received.
   * @param msg - The RequestMessage object containing the message ID and payload.
   */
  private async handleSocketMessage(socket: Socket, { msgId, payload }: RequestMessage<Payload>) {
    try {
      const data = await this.msgHandlerFn(payload);

      const [respPayload, transferables] = this.getPayloadAndTransfers(data);
      const rep: ResponseMessage<Payload> = { msgId, payload: respPayload };

      await socket.send(rep, transferables);
    } catch (err: any) {
      const rep: ResponseMessage<Payload> = { msgId, error: err.stack };
      await socket.send(rep);
    }
  }
}
