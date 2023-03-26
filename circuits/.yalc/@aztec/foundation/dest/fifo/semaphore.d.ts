/**
 * Allows the acquiring of up to `size` tokens before calls to acquire block, waiting for a call to release().
 */
export declare class Semaphore {
    private readonly queue;
    constructor(size: number);
    acquire(): Promise<void>;
    release(): void;
}
//# sourceMappingURL=semaphore.d.ts.map