/**
 *
 */
export interface DispatchMsg {
    /**
     *
     */
    fn: string;
    /**
     *
     */
    args: any[];
}
/**
 *
 */
export declare function createDispatchFn(targetFn: () => any, debug?: {
    (...data: any[]): void;
    (message?: any, ...optionalParams: any[]): void;
}): ({ fn, args }: DispatchMsg) => Promise<any>;
//# sourceMappingURL=create_dispatch_fn.d.ts.map