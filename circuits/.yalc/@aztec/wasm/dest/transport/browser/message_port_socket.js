/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class MessagePortSocket {
    /**
     * Create a MessagePortSocket.
     * @param port - MessagePort object to wrap.
     */
    constructor(port) {
        this.port = port;
    }
    /**
     * Send a message over our message port.
     * @param msg - The message.
     * @param transfer - Objects to transfer ownership of.
     */
    send(msg, transfer = []) {
        this.port.postMessage(msg, transfer);
        return Promise.resolve();
    }
    /**
     * Add a message handler.
     * @param cb - The handler.
     */
    registerHandler(cb) {
        this.port.onmessage = event => cb(event.data);
    }
    /**
     * Close this message port.
     */
    close() {
        void this.send(undefined);
        this.port.onmessage = null;
        this.port.close();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWVzc2FnZV9wb3J0X3NvY2tldC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci9tZXNzYWdlX3BvcnRfc29ja2V0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUVBOztHQUVHO0FBQ0gsTUFBTSxPQUFPLGlCQUFpQjtJQUM1Qjs7O09BR0c7SUFDSCxZQUFvQixJQUFpQjtRQUFqQixTQUFJLEdBQUosSUFBSSxDQUFhO0lBQUcsQ0FBQztJQUV6Qzs7OztPQUlHO0lBQ0gsSUFBSSxDQUFDLEdBQVEsRUFBRSxXQUEyQixFQUFFO1FBQzFDLElBQUksQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDLEdBQUcsRUFBRSxRQUFRLENBQUMsQ0FBQztRQUNyQyxPQUFPLE9BQU8sQ0FBQyxPQUFPLEVBQUUsQ0FBQztJQUMzQixDQUFDO0lBRUQ7OztPQUdHO0lBQ0gsZUFBZSxDQUFDLEVBQXFCO1FBQ25DLElBQUksQ0FBQyxJQUFJLENBQUMsU0FBUyxHQUFHLEtBQUssQ0FBQyxFQUFFLENBQUMsRUFBRSxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsQ0FBQztJQUNoRCxDQUFDO0lBRUQ7O09BRUc7SUFDSCxLQUFLO1FBQ0gsS0FBSyxJQUFJLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxDQUFDO1FBQzFCLElBQUksQ0FBQyxJQUFJLENBQUMsU0FBUyxHQUFHLElBQUksQ0FBQztRQUMzQixJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQ3BCLENBQUM7Q0FDRiJ9