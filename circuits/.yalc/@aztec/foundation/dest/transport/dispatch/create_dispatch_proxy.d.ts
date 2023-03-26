import { DispatchMsg } from './create_dispatch_fn.js';
import { TransportClient } from '../transport_client.js';
import { TransferDescriptor } from '../interface/transferable.js';
type FilterOutAttributes<Base> = {
    [Key in keyof Base]: Base[Key] extends (...any: any) => any ? Base[Key] : never;
};
type PromisifyFunction<F extends (...any: any) => any> = (...args: Parameters<F>) => Promise<ReturnType<F>>;
type Promisify<Base extends {
    [key: string]: (...any: any) => any;
}> = {
    [Key in keyof Base]: ReturnType<Base[Key]> extends Promise<any> ? Base[Key] : PromisifyFunction<Base[Key]>;
};
type TransferTypes<Tuple extends [...args: any]> = {
    [Index in keyof Tuple]: Tuple[Index] | (Tuple[Index] extends Transferable ? TransferDescriptor<Tuple[Index]> : never);
};
/**
 * Annoying: https://github.com/microsoft/TypeScript/issues/29919
 * There's a bug that means we can't map over the tuple or function parameter types to make them transferrable, if
 * we use the Parameters builtin, and then try to map.
 * So instead we inline the Parameters builtin and apply the TransferTypes to the parameters within the inline.
 * Once the above is fixed we could in theory just do:
 *
 * type MakeFunctionTransferrable<TFunction extends (...args: any) => any> = (
 *   ...args: TransferTypes<Parameters<TFunction>>
 * ) => ReturnType<TFunction>;
 */
type MakeFunctionTransferrable<TFunction extends (...args: any) => any> = (...args: TFunction extends (...args: infer P) => any ? TransferTypes<P> : never) => ReturnType<TFunction>;
type Transferrable<Base extends {
    [key: string]: (...any: any) => any;
}> = {
    [Key in keyof Base]: MakeFunctionTransferrable<Base[Key]>;
};
export type Proxify<T> = Promisify<Transferrable<FilterOutAttributes<T>>>;
export declare function createDispatchProxyFromFn<T>(class_: {
    new (...args: any[]): T;
}, requestFn: (fn: string) => (...args: any[]) => Promise<any>): Proxify<T>;
export declare function createDispatchProxy<T>(class_: {
    new (...args: any[]): T;
}, transportClient: TransportClient<DispatchMsg>): Proxify<T>;
export {};
//# sourceMappingURL=create_dispatch_proxy.d.ts.map