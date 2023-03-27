import { isTransferDescriptor } from './interface/transferable.js';
/**
 * Keeps track of clients, providing a broadcast, and request/response api with multiplexing.
 */
export class TransportServer {
    constructor(listener, msgHandlerFn) {
        this.listener = listener;
        this.msgHandlerFn = msgHandlerFn;
        this.sockets = [];
    }
    /**
     * Start accepting new connections.
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
     * Broadcast a message.
     * @param msg - The message.
     */
    async broadcast(msg) {
        await Promise.all(this.sockets.map(s => s.send({ payload: msg })));
    }
    /**
     * New socket registration.
     * @param socket - The socket to register.
     */
    handleNewSocket(socket) {
        socket.registerHandler(async (msg) => {
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
     * @param data - The return object.
     * @returns - The data and the.
     */
    getPayloadAndTransfers(data) {
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
     * Handles a socket message from a listener.
     * @param socket - The socket.
     * @param requestMessage - The message to handle.
     * @returns The socket response.
     */
    async handleSocketMessage(socket, { msgId, payload }) {
        try {
            const data = await this.msgHandlerFn(payload);
            const [respPayload, transferables] = this.getPayloadAndTransfers(data);
            const rep = { msgId, payload: respPayload };
            await socket.send(rep, transferables);
        }
        catch (err) {
            const rep = { msgId, error: err.stack };
            await socket.send(rep);
        }
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidHJhbnNwb3J0X3NlcnZlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy90cmFuc3BvcnQvdHJhbnNwb3J0X3NlcnZlci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFHQSxPQUFPLEVBQUUsb0JBQW9CLEVBQUUsTUFBTSw2QkFBNkIsQ0FBQztBQUVuRTs7R0FFRztBQUNILE1BQU0sT0FBTyxlQUFlO0lBRzFCLFlBQW9CLFFBQWtCLEVBQVUsWUFBNEM7UUFBeEUsYUFBUSxHQUFSLFFBQVEsQ0FBVTtRQUFVLGlCQUFZLEdBQVosWUFBWSxDQUFnQztRQUZwRixZQUFPLEdBQWEsRUFBRSxDQUFDO0lBRWdFLENBQUM7SUFFaEc7O09BRUc7SUFDSCxLQUFLO1FBQ0gsSUFBSSxDQUFDLFFBQVEsQ0FBQyxFQUFFLENBQUMsWUFBWSxFQUFFLE1BQU0sQ0FBQyxFQUFFLENBQUMsSUFBSSxDQUFDLGVBQWUsQ0FBQyxNQUFNLENBQUMsQ0FBQyxDQUFDO1FBQ3ZFLElBQUksQ0FBQyxRQUFRLENBQUMsSUFBSSxFQUFFLENBQUM7SUFDdkIsQ0FBQztJQUVEOzs7T0FHRztJQUNILElBQUk7UUFDRixJQUFJLENBQUMsUUFBUSxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQ3hCLENBQUM7SUFFRDs7O09BR0c7SUFDSCxLQUFLLENBQUMsU0FBUyxDQUFDLEdBQVk7UUFDMUIsTUFBTSxPQUFPLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxPQUFPLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQyxFQUFFLE9BQU8sRUFBRSxHQUFHLEVBQUUsQ0FBQyxDQUFDLENBQUMsQ0FBQztJQUNyRSxDQUFDO0lBRUQ7OztPQUdHO0lBQ0ssZUFBZSxDQUFDLE1BQWM7UUFDcEMsTUFBTSxDQUFDLGVBQWUsQ0FBQyxLQUFLLEVBQUMsR0FBRyxFQUFDLEVBQUU7WUFDakMsSUFBSSxHQUFHLEtBQUssU0FBUyxFQUFFO2dCQUNyQixrR0FBa0c7Z0JBQ2xHLE1BQU0sV0FBVyxHQUFHLElBQUksQ0FBQyxPQUFPLENBQUMsU0FBUyxDQUFDLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxLQUFLLE1BQU0sQ0FBQyxDQUFDO2dCQUM5RCxNQUFNLENBQUMsYUFBYSxDQUFDLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBQyxNQUFNLENBQUMsV0FBVyxFQUFFLENBQUMsQ0FBQyxDQUFDO2dCQUM1RCxhQUFhLENBQUMsS0FBSyxFQUFFLENBQUM7Z0JBQ3RCLE9BQU87YUFDUjtZQUNELE9BQU8sTUFBTSxJQUFJLENBQUMsbUJBQW1CLENBQUMsTUFBTSxFQUFFLEdBQUcsQ0FBQyxDQUFDO1FBQ3JELENBQUMsQ0FBQyxDQUFDO1FBQ0gsSUFBSSxDQUFDLE9BQU8sQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDLENBQUM7SUFDNUIsQ0FBQztJQUVEOzs7OztPQUtHO0lBQ0ssc0JBQXNCLENBQUMsSUFBUztRQUN0QyxJQUFJLG9CQUFvQixDQUFDLElBQUksQ0FBQyxFQUFFO1lBQzlCLGdFQUFnRTtZQUNoRSxzRUFBc0U7WUFDdEUsT0FBTyxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsSUFBSSxDQUFDLGFBQWEsQ0FBQyxDQUFDO1NBQ3hDO1FBQ0QsSUFBSSxJQUFJLFlBQVksVUFBVSxFQUFFO1lBQzlCLDRHQUE0RztZQUM1Ryw0R0FBNEc7WUFDNUcsTUFBTSxXQUFXLEdBQUcsSUFBSSxZQUFZLFVBQVUsSUFBSSxXQUFXLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBQyxJQUFJLFVBQVUsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDLENBQUMsSUFBSSxDQUFDO1lBQ3pHLE1BQU0sYUFBYSxHQUFHLElBQUksWUFBWSxVQUFVLENBQUMsQ0FBQyxDQUFDLENBQUMsV0FBVyxDQUFDLE1BQU0sQ0FBQyxDQUFDLENBQUMsQ0FBQyxFQUFFLENBQUM7WUFDN0UsT0FBTyxDQUFDLFdBQVcsRUFBRSxhQUFhLENBQUMsQ0FBQztTQUNyQztRQUNELE9BQU8sQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDcEIsQ0FBQztJQUNEOzs7OztPQUtHO0lBQ0ssS0FBSyxDQUFDLG1CQUFtQixDQUFDLE1BQWMsRUFBRSxFQUFFLEtBQUssRUFBRSxPQUFPLEVBQTJCO1FBQzNGLElBQUk7WUFDRixNQUFNLElBQUksR0FBRyxNQUFNLElBQUksQ0FBQyxZQUFZLENBQUMsT0FBTyxDQUFDLENBQUM7WUFFOUMsTUFBTSxDQUFDLFdBQVcsRUFBRSxhQUFhLENBQUMsR0FBRyxJQUFJLENBQUMsc0JBQXNCLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDdkUsTUFBTSxHQUFHLEdBQTZCLEVBQUUsS0FBSyxFQUFFLE9BQU8sRUFBRSxXQUFXLEVBQUUsQ0FBQztZQUV0RSxNQUFNLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxFQUFFLGFBQWEsQ0FBQyxDQUFDO1NBQ3ZDO1FBQUMsT0FBTyxHQUFRLEVBQUU7WUFDakIsTUFBTSxHQUFHLEdBQTZCLEVBQUUsS0FBSyxFQUFFLEtBQUssRUFBRSxHQUFHLENBQUMsS0FBSyxFQUFFLENBQUM7WUFDbEUsTUFBTSxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO1NBQ3hCO0lBQ0gsQ0FBQztDQUNGIn0=