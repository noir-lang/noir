/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class MessagePortSocket {
    constructor(port) {
        this.port = port;
    }
    send(msg, transfer = []) {
        this.port.postMessage(msg, transfer);
        return Promise.resolve();
    }
    registerHandler(cb) {
        this.port.onmessage = event => cb(event.data);
    }
    close() {
        void this.send(undefined);
        this.port.onmessage = null;
        this.port.close();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWVzc2FnZV9wb3J0X3NvY2tldC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy90cmFuc3BvcnQvYnJvd3Nlci9tZXNzYWdlX3BvcnRfc29ja2V0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUVBOztHQUVHO0FBQ0gsTUFBTSxPQUFPLGlCQUFpQjtJQUM1QixZQUFvQixJQUFpQjtRQUFqQixTQUFJLEdBQUosSUFBSSxDQUFhO0lBQUcsQ0FBQztJQUV6QyxJQUFJLENBQUMsR0FBUSxFQUFFLFdBQTJCLEVBQUU7UUFDMUMsSUFBSSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQUMsR0FBRyxFQUFFLFFBQVEsQ0FBQyxDQUFDO1FBQ3JDLE9BQU8sT0FBTyxDQUFDLE9BQU8sRUFBRSxDQUFDO0lBQzNCLENBQUM7SUFFRCxlQUFlLENBQUMsRUFBcUI7UUFDbkMsSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLEdBQUcsS0FBSyxDQUFDLEVBQUUsQ0FBQyxFQUFFLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDO0lBQ2hELENBQUM7SUFFRCxLQUFLO1FBQ0gsS0FBSyxJQUFJLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxDQUFDO1FBQzFCLElBQUksQ0FBQyxJQUFJLENBQUMsU0FBUyxHQUFHLElBQUksQ0FBQztRQUMzQixJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQ3BCLENBQUM7Q0FDRiJ9