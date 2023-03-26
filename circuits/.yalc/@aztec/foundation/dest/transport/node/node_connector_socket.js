export class NodeConnectorSocket {
    constructor(worker) {
        this.worker = worker;
    }
    send(msg, transfer = []) {
        this.worker.postMessage(msg, transfer);
        return Promise.resolve();
    }
    registerHandler(cb) {
        this.worker.on('message', cb);
    }
    close() {
        void this.send(undefined);
        this.worker.removeAllListeners();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9jb25uZWN0b3Jfc29ja2V0LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3RyYW5zcG9ydC9ub2RlL25vZGVfY29ubmVjdG9yX3NvY2tldC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFHQSxNQUFNLE9BQU8sbUJBQW1CO0lBQzlCLFlBQW9CLE1BQWM7UUFBZCxXQUFNLEdBQU4sTUFBTSxDQUFRO0lBQUcsQ0FBQztJQUV0QyxJQUFJLENBQUMsR0FBUSxFQUFFLFdBQTJCLEVBQUU7UUFDMUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxXQUFXLENBQUMsR0FBRyxFQUFFLFFBQThCLENBQUMsQ0FBQztRQUM3RCxPQUFPLE9BQU8sQ0FBQyxPQUFPLEVBQUUsQ0FBQztJQUMzQixDQUFDO0lBRUQsZUFBZSxDQUFDLEVBQXFCO1FBQ25DLElBQUksQ0FBQyxNQUFNLENBQUMsRUFBRSxDQUFDLFNBQVMsRUFBRSxFQUFFLENBQUMsQ0FBQztJQUNoQyxDQUFDO0lBRUQsS0FBSztRQUNILEtBQUssSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsQ0FBQztRQUMxQixJQUFJLENBQUMsTUFBTSxDQUFDLGtCQUFrQixFQUFFLENBQUM7SUFDbkMsQ0FBQztDQUNGIn0=