/**
 * An implementation of a TransportSocket using MessagePorts.
 */
export class NodeListenerSocket {
    constructor(port) {
        this.port = port;
    }
    send(msg, transfer = []) {
        this.port.postMessage(msg, transfer);
        return Promise.resolve();
    }
    registerHandler(cb) {
        this.port.on('message', cb);
    }
    close() {
        void this.send(undefined);
        this.port.removeAllListeners();
        this.port.close();
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9saXN0ZW5lcl9zb2NrZXQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvdHJhbnNwb3J0L25vZGUvbm9kZV9saXN0ZW5lcl9zb2NrZXQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBR0E7O0dBRUc7QUFDSCxNQUFNLE9BQU8sa0JBQWtCO0lBQzdCLFlBQW9CLElBQWlCO1FBQWpCLFNBQUksR0FBSixJQUFJLENBQWE7SUFBRyxDQUFDO0lBRXpDLElBQUksQ0FBQyxHQUFRLEVBQUUsV0FBMkIsRUFBRTtRQUMxQyxJQUFJLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQyxHQUFHLEVBQUUsUUFBOEIsQ0FBQyxDQUFDO1FBQzNELE9BQU8sT0FBTyxDQUFDLE9BQU8sRUFBRSxDQUFDO0lBQzNCLENBQUM7SUFFRCxlQUFlLENBQUMsRUFBcUI7UUFDbkMsSUFBSSxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsU0FBUyxFQUFFLEVBQUUsQ0FBQyxDQUFDO0lBQzlCLENBQUM7SUFFRCxLQUFLO1FBQ0gsS0FBSyxJQUFJLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxDQUFDO1FBQzFCLElBQUksQ0FBQyxJQUFJLENBQUMsa0JBQWtCLEVBQUUsQ0FBQztRQUMvQixJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQ3BCLENBQUM7Q0FDRiJ9