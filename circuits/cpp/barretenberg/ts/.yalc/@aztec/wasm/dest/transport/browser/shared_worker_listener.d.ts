/// <reference types="node" resolution-mode="require"/>
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
/**
 * See https://developer.mozilla.org/en-US/docs/Web/API/SharedWorkerGlobalScope.
 */
declare interface SharedWorkerGlobalScope {
    /**
     * Fired on shared workers when a new client connects.
     */
    onconnect: any;
}
/**
 * Listens for connections to a shared worker.
 */
export declare class SharedWorkerListener extends EventEmitter implements Listener {
    private worker;
    /**
     *
     * @param worker
     */
    constructor(worker: SharedWorkerGlobalScope);
    /**
     *
     */
    open(): void;
    /**
     *
     */
    close(): void;
    /**
     *
     * @param event
     */
    private handleMessageEvent;
}
export {};
//# sourceMappingURL=shared_worker_listener.d.ts.map