import { Socket } from './socket.js';
/**
 * Opens a socket with corresponding TransportListener.
 */
export interface Connector {
    createSocket(): Promise<Socket>;
}
//# sourceMappingURL=connector.d.ts.map