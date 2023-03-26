/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class NodeListenerSocket {
    constructor(port) {
        this.port = port;
    }
    /**
     * Send a message over this port.
     * @param msg - The message.
     * @param transfer - Transferable objects.
     * @returns A void promise.
     */
    send(msg, transfer = []) {
        this.port.postMessage(msg, transfer);
        return Promise.resolve();
    }
    /**
     * Add a handler to this port.
     * @param cb - The handler function.
     */
    registerHandler(cb) {
        this.port.on('message', cb);
    }
    /**
     * Close this socket.
     */
    close() {
        void this.send(undefined);
        this.port.removeAllListeners();
        this.port.close();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9saXN0ZW5lcl9zb2NrZXQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvdHJhbnNwb3J0L25vZGUvbm9kZV9saXN0ZW5lcl9zb2NrZXQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBR0E7O0dBRUc7QUFDSCxNQUFNLE9BQU8sa0JBQWtCO0lBQzdCLFlBQW9CLElBQWlCO1FBQWpCLFNBQUksR0FBSixJQUFJLENBQWE7SUFBRyxDQUFDO0lBRXpDOzs7OztPQUtHO0lBQ0gsSUFBSSxDQUFDLEdBQVEsRUFBRSxXQUEyQixFQUFFO1FBQzFDLElBQUksQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDLEdBQUcsRUFBRSxRQUE4QixDQUFDLENBQUM7UUFDM0QsT0FBTyxPQUFPLENBQUMsT0FBTyxFQUFFLENBQUM7SUFDM0IsQ0FBQztJQUVEOzs7T0FHRztJQUNILGVBQWUsQ0FBQyxFQUFxQjtRQUNuQyxJQUFJLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxTQUFTLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDOUIsQ0FBQztJQUVEOztPQUVHO0lBQ0gsS0FBSztRQUNILEtBQUssSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsQ0FBQztRQUMxQixJQUFJLENBQUMsSUFBSSxDQUFDLGtCQUFrQixFQUFFLENBQUM7UUFDL0IsSUFBSSxDQUFDLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUNwQixDQUFDO0NBQ0YifQ==