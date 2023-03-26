export declare function backoffGenerator(): Generator<number, void, unknown>;
export declare function retry<Result>(fn: () => Promise<Result>, name?: string, backoff?: Generator<number, void, unknown>): Promise<Result>;
export declare function retryUntil(fn: () => boolean | Promise<boolean>, name?: string, timeout?: number, interval?: number): Promise<void>;
//# sourceMappingURL=index.d.ts.map