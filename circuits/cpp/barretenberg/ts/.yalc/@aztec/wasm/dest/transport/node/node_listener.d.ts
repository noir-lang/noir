/// <reference types="node" resolution-mode="require"/>
import EventEmitter from 'events';
import { Listener } from '../interface/listener.js';
/**
 * A socket listener that works with Node.
 */
export declare class NodeListener extends EventEmitter implements Listener {
    constructor();
    /**
     * Open the listener.
     */
    open(): void;
    /**
     * Close the listener.
     */
    close(): void;
}
//# sourceMappingURL=node_listener.d.ts.map