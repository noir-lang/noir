/// <reference types="node" resolution-mode="require"/>
import EventEmitter from 'events';
import { Connector } from './interface/connector.js';
export interface TransportClient<Payload> extends EventEmitter {
    on(name: 'event_msg', handler: (payload: Payload) => void): this;
    emit(name: 'event_msg', payload: Payload): boolean;
}
/**
 * A TransportClient provides a request/response and event api to a corresponding TransportServer.
 * If `broadcast` is called on TransportServer, TransportClients will emit an `event_msg`.
 * The `request` method will block until a response is returned from the TransportServer's dispatch function.
 * Request multiplexing is supported.
 */
export declare class TransportClient<Payload> extends EventEmitter {
    private transportConnect;
    private msgId;
    private pendingRequests;
    private socket?;
    constructor(transportConnect: Connector);
    open(): Promise<void>;
    close(): void;
    request(payload: Payload, transfer?: Transferable[]): Promise<any>;
    private handleSocketMessage;
}
//# sourceMappingURL=transport_client.d.ts.map