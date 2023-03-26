/**
 * A socket implementation using a Node worker.
 */
export class NodeConnectorSocket {
    constructor(worker) {
        this.worker = worker;
    }
    /**
     * Send a message.
     * @param msg - The message.
     * @param transfer - Objects to transfer ownership of.
     * @returns A void promise.
     */
    send(msg, transfer = []) {
        this.worker.postMessage(msg, transfer);
        return Promise.resolve();
    }
    /**
     * Register a message handler.
     * @param cb - The handler function.
     */
    registerHandler(cb) {
        this.worker.on('message', cb);
    }
    /**
     * Remove all listeners from our worker.
     */
    close() {
        void this.send(undefined);
        this.worker.removeAllListeners();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9jb25uZWN0b3Jfc29ja2V0LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9ub2RlL25vZGVfY29ubmVjdG9yX3NvY2tldC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFHQTs7R0FFRztBQUNILE1BQU0sT0FBTyxtQkFBbUI7SUFDOUIsWUFBb0IsTUFBYztRQUFkLFdBQU0sR0FBTixNQUFNLENBQVE7SUFBRyxDQUFDO0lBRXRDOzs7OztPQUtHO0lBQ0gsSUFBSSxDQUFDLEdBQVEsRUFBRSxXQUEyQixFQUFFO1FBQzFDLElBQUksQ0FBQyxNQUFNLENBQUMsV0FBVyxDQUFDLEdBQUcsRUFBRSxRQUE4QixDQUFDLENBQUM7UUFDN0QsT0FBTyxPQUFPLENBQUMsT0FBTyxFQUFFLENBQUM7SUFDM0IsQ0FBQztJQUVEOzs7T0FHRztJQUNILGVBQWUsQ0FBQyxFQUFxQjtRQUNuQyxJQUFJLENBQUMsTUFBTSxDQUFDLEVBQUUsQ0FBQyxTQUFTLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDaEMsQ0FBQztJQUVEOztPQUVHO0lBQ0gsS0FBSztRQUNILEtBQUssSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsQ0FBQztRQUMxQixJQUFJLENBQUMsTUFBTSxDQUFDLGtCQUFrQixFQUFFLENBQUM7SUFDbkMsQ0FBQztDQUNGIn0=