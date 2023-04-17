import { NodeConnectorSocket } from './node_connector_socket.js';
/**
 * Creates sockets backed by a Node worker.
 */
export class NodeConnector {
    constructor(worker) {
        this.worker = worker;
    }
    /**
     * Creates a socket backed by a node worker.
     * @returns The socket.
     */
    createSocket() {
        return Promise.resolve(new NodeConnectorSocket(this.worker));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9jb25uZWN0b3IuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvdHJhbnNwb3J0L25vZGUvbm9kZV9jb25uZWN0b3IudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsT0FBTyxFQUFFLG1CQUFtQixFQUFFLE1BQU0sNEJBQTRCLENBQUM7QUFFakU7O0dBRUc7QUFDSCxNQUFNLE9BQU8sYUFBYTtJQUN4QixZQUFvQixNQUFjO1FBQWQsV0FBTSxHQUFOLE1BQU0sQ0FBUTtJQUFHLENBQUM7SUFFdEM7OztPQUdHO0lBQ0gsWUFBWTtRQUNWLE9BQU8sT0FBTyxDQUFDLE9BQU8sQ0FBQyxJQUFJLG1CQUFtQixDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBQyxDQUFDO0lBQy9ELENBQUM7Q0FDRiJ9