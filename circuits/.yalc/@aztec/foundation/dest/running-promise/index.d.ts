export declare class RunningPromise {
    private fn;
    private pollingInterval;
    private running;
    private runningPromise;
    private interruptPromise;
    private interruptResolve;
    constructor(fn: () => Promise<void>, pollingInterval?: number);
    /**
     * Starts the running promise
     */
    start(): void;
    stop(): Promise<void>;
    private interruptableSleep;
    isRunning(): boolean;
}
//# sourceMappingURL=index.d.ts.map