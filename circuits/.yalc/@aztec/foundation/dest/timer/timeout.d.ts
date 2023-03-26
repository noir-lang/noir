export declare class TimeoutTask<T> {
    private fn;
    private timeout;
    private interruptPromise;
    private interrupt;
    private totalTime;
    constructor(fn: () => Promise<T>, timeout?: number, fnName?: string);
    exec(): Promise<T>;
    getInterruptPromise(): Promise<any>;
    getTime(): number;
}
export declare const executeTimeout: <T>(fn: () => Promise<T>, timeout?: number, fnName?: string) => Promise<T>;
//# sourceMappingURL=timeout.d.ts.map