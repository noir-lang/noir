import { MessagePortSocket } from './message_port_socket.js';
/**
 * Connector implementation which wraps a SharedWorker.
 */
export class SharedWorkerConnector {
    /**
     * Create a SharedWorkerConnector.
     * @param worker - A shared worker.
     */
    constructor(worker) {
        this.worker = worker;
    }
    /**
     * Create a Socket implementation with our mesage port.
     * @returns The socket.
     */
    createSocket() {
        return Promise.resolve(new MessagePortSocket(this.worker.port));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2hhcmVkX3dvcmtlcl9jb25uZWN0b3IuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvdHJhbnNwb3J0L2Jyb3dzZXIvc2hhcmVkX3dvcmtlcl9jb25uZWN0b3IudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQ0EsT0FBTyxFQUFFLGlCQUFpQixFQUFFLE1BQU0sMEJBQTBCLENBQUM7QUFFN0Q7O0dBRUc7QUFDSCxNQUFNLE9BQU8scUJBQXFCO0lBQ2hDOzs7T0FHRztJQUNILFlBQW9CLE1BQW9CO1FBQXBCLFdBQU0sR0FBTixNQUFNLENBQWM7SUFBRyxDQUFDO0lBRTVDOzs7T0FHRztJQUNILFlBQVk7UUFDVixPQUFPLE9BQU8sQ0FBQyxPQUFPLENBQUMsSUFBSSxpQkFBaUIsQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7SUFDbEUsQ0FBQztDQUNGIn0=