/// <reference types="node" resolution-mode="require"/>
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
declare interface SharedWorkerGlobalScope {
    onconnect: (...args: any) => any;
}
export declare class SharedWorkerListener extends EventEmitter implements Listener {
    private worker;
    constructor(worker: SharedWorkerGlobalScope);
    open(): void;
    close(): void;
    private handleMessageEvent;
}
export {};
//# sourceMappingURL=shared_worker_listener.d.ts.map