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
    async broadcast(msg) {
        await Promise.all(this.sockets.map(s => s.send({ payload: msg })));
    }
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidHJhbnNwb3J0X3NlcnZlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy90cmFuc3BvcnQvdHJhbnNwb3J0X3NlcnZlci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFHQSxPQUFPLEVBQUUsb0JBQW9CLEVBQUUsTUFBTSw2QkFBNkIsQ0FBQztBQUVuRTs7R0FFRztBQUNILE1BQU0sT0FBTyxlQUFlO0lBRzFCLFlBQW9CLFFBQWtCLEVBQVUsWUFBNEM7UUFBeEUsYUFBUSxHQUFSLFFBQVEsQ0FBVTtRQUFVLGlCQUFZLEdBQVosWUFBWSxDQUFnQztRQUZwRixZQUFPLEdBQWEsRUFBRSxDQUFDO0lBRWdFLENBQUM7SUFFaEcsS0FBSztRQUNILElBQUksQ0FBQyxRQUFRLENBQUMsRUFBRSxDQUFDLFlBQVksRUFBRSxNQUFNLENBQUMsRUFBRSxDQUFDLElBQUksQ0FBQyxlQUFlLENBQUMsTUFBTSxDQUFDLENBQUMsQ0FBQztRQUN2RSxJQUFJLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxDQUFDO0lBQ3ZCLENBQUM7SUFFRDs7O09BR0c7SUFDSCxJQUFJO1FBQ0YsSUFBSSxDQUFDLFFBQVEsQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUN4QixDQUFDO0lBRUQsS0FBSyxDQUFDLFNBQVMsQ0FBQyxHQUFZO1FBQzFCLE1BQU0sT0FBTyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQUMsRUFBRSxPQUFPLEVBQUUsR0FBRyxFQUFFLENBQUMsQ0FBQyxDQUFDLENBQUM7SUFDckUsQ0FBQztJQUVPLGVBQWUsQ0FBQyxNQUFjO1FBQ3BDLE1BQU0sQ0FBQyxlQUFlLENBQUMsS0FBSyxFQUFDLEdBQUcsRUFBQyxFQUFFO1lBQ2pDLElBQUksR0FBRyxLQUFLLFNBQVMsRUFBRTtnQkFDckIsa0dBQWtHO2dCQUNsRyxNQUFNLFdBQVcsR0FBRyxJQUFJLENBQUMsT0FBTyxDQUFDLFNBQVMsQ0FBQyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsS0FBSyxNQUFNLENBQUMsQ0FBQztnQkFDOUQsTUFBTSxDQUFDLGFBQWEsQ0FBQyxHQUFHLElBQUksQ0FBQyxPQUFPLENBQUMsTUFBTSxDQUFDLFdBQVcsRUFBRSxDQUFDLENBQUMsQ0FBQztnQkFDNUQsYUFBYSxDQUFDLEtBQUssRUFBRSxDQUFDO2dCQUN0QixPQUFPO2FBQ1I7WUFDRCxPQUFPLE1BQU0sSUFBSSxDQUFDLG1CQUFtQixDQUFDLE1BQU0sRUFBRSxHQUFHLENBQUMsQ0FBQztRQUNyRCxDQUFDLENBQUMsQ0FBQztRQUNILElBQUksQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDO0lBQzVCLENBQUM7SUFFRDs7O09BR0c7SUFDSyxzQkFBc0IsQ0FBQyxJQUFTO1FBQ3RDLElBQUksb0JBQW9CLENBQUMsSUFBSSxDQUFDLEVBQUU7WUFDOUIsZ0VBQWdFO1lBQ2hFLHNFQUFzRTtZQUN0RSxPQUFPLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxJQUFJLENBQUMsYUFBYSxDQUFDLENBQUM7U0FDeEM7UUFDRCxJQUFJLElBQUksWUFBWSxVQUFVLEVBQUU7WUFDOUIsNEdBQTRHO1lBQzVHLDRHQUE0RztZQUM1RyxNQUFNLFdBQVcsR0FBRyxJQUFJLFlBQVksVUFBVSxJQUFJLFdBQVcsQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQyxDQUFDLElBQUksVUFBVSxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQUM7WUFDekcsTUFBTSxhQUFhLEdBQUcsSUFBSSxZQUFZLFVBQVUsQ0FBQyxDQUFDLENBQUMsQ0FBQyxXQUFXLENBQUMsTUFBTSxDQUFDLENBQUMsQ0FBQyxDQUFDLEVBQUUsQ0FBQztZQUM3RSxPQUFPLENBQUMsV0FBVyxFQUFFLGFBQWEsQ0FBQyxDQUFDO1NBQ3JDO1FBQ0QsT0FBTyxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsQ0FBQztJQUNwQixDQUFDO0lBQ08sS0FBSyxDQUFDLG1CQUFtQixDQUFDLE1BQWMsRUFBRSxFQUFFLEtBQUssRUFBRSxPQUFPLEVBQTJCO1FBQzNGLElBQUk7WUFDRixNQUFNLElBQUksR0FBRyxNQUFNLElBQUksQ0FBQyxZQUFZLENBQUMsT0FBTyxDQUFDLENBQUM7WUFFOUMsTUFBTSxDQUFDLFdBQVcsRUFBRSxhQUFhLENBQUMsR0FBRyxJQUFJLENBQUMsc0JBQXNCLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDdkUsTUFBTSxHQUFHLEdBQTZCLEVBQUUsS0FBSyxFQUFFLE9BQU8sRUFBRSxXQUFXLEVBQUUsQ0FBQztZQUV0RSxNQUFNLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxFQUFFLGFBQWEsQ0FBQyxDQUFDO1NBQ3ZDO1FBQUMsT0FBTyxHQUFRLEVBQUU7WUFDakIsTUFBTSxHQUFHLEdBQTZCLEVBQUUsS0FBSyxFQUFFLEtBQUssRUFBRSxHQUFHLENBQUMsS0FBSyxFQUFFLENBQUM7WUFDbEUsTUFBTSxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO1NBQ3hCO0lBQ0gsQ0FBQztDQUNGIn0=