/// <reference types="node" resolution-mode="require"/>
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
export declare class NodeListener extends EventEmitter implements Listener {
    constructor();
    open(): void;
    close(): void;
}
//# sourceMappingURL=node_listener.d.ts.map