import { EventEmitter } from 'events';

import { type TransferDescriptor, isTransferDescriptor } from '../interface/transferable.js';
import { type TransportClient } from '../transport_client.js';
import { type DispatchMsg } from './create_dispatch_fn.js';

/**
 * FilterOutAttributes type filters out all non-method properties of an object, leaving only the attributes
 * that are functions. This is useful for creating proxies or wrappers around objects while focusing only
 * on their methods and ignoring other properties.
 */
type FilterOutAttributes<Base> = {
  [Key in keyof Base]: Base[Key] extends (...any: any) => any ? Base[Key] : never;
};

/**
 * Takes a function type `F` and returns a new function type with the same input parameters as `F`,
 * but returning a Promise of the original return type of `F`. This is useful for converting sync
 * functions or functions that take callbacks into a version that returns a Promise.
 */
type PromisifyFunction<F extends (...any: any) => any> = (...args: Parameters<F>) => Promise<ReturnType<F>>;

/**
 * Transforms the provided object type by converting each of its function properties into their
 * promise-returning counterparts. If a function property already returns a promise, it remains unchanged.
 * This is useful when wrapping synchronous methods to return promises in order to standardize the API for
 * asynchronous operations.
 *
 * @typeParam Base - The type of the object whose function properties need to be converted into their
 *                   promise-returning versions.
 */
type Promisify<Base extends { [key: string]: (...any: any) => any }> = {
  [Key in keyof Base]: ReturnType<Base[Key]> extends Promise<any> ? Base[Key] : PromisifyFunction<Base[Key]>;
};

/**
 * Type that transforms a tuple of types, replacing each type 'T' with either 'T' or a `TransferDescriptor<T>` if 'T' is `Transferable`.
 * This is useful for handling arguments of functions that may accept both original and transferable representations of objects.
 */
type TransferTypes<Tuple extends [...args: any]> = {
  [Index in keyof Tuple]: Tuple[Index] | (Tuple[Index] extends Transferable ? TransferDescriptor<Tuple[Index]> : never);
};

/**
 * Annoying.
 * @see https://github.com/microsoft/TypeScript/issues/29919
 * There's a bug that means we can't map over the tuple or function parameter types to make them transferrable, if
 * we use the Parameters builtin, and then try to map.
 * So instead we inline the Parameters builtin and apply the TransferTypes to the parameters within the inline.
 * Once the above is fixed we could in theory just do:
 *
 * type MakeFunctionTransferrable\<TFunction extends (...args: any) =\> any\> = (
 *   ...args: TransferTypes\<Parameters\<TFunction\>\>
 * ) =\> ReturnType<TFunction>;
 */
type MakeFunctionTransferrable<TFunction extends (...args: any) => any> = (
  ...args: TFunction extends (...args: infer P) => any ? TransferTypes<P> : never
) => ReturnType<TFunction>;

/**
 * Transferrable type represents a utility type that maps over the provided Base object's methods,
 * transforming their argument types to support transferable objects. This is useful when dealing
 * with operations across different environments or threads, such as Web Workers or Node.js processes,
 * where certain objects need to be transferred instead of being serialized and deserialized.
 */
type Transferrable<Base extends { [key: string]: (...any: any) => any }> = {
  [Key in keyof Base]: MakeFunctionTransferrable<Base[Key]>;
};

/**
 * Proxify is a mapped type that takes an object with functions as its properties and returns
 * a new object with the same properties, but with each function transformed to return a Promise
 * and accept Transferable types in place of their original parameters. This type is useful for
 * creating proxies that communicate over different environments or threads while maintaining
 * the original class's method signatures, allowing for type-safe interaction with remote instances.
 */
export type Proxify<T> = Promisify<Transferrable<FilterOutAttributes<T>>>;

/**
 * Creates a proxy object for the provided class, wrapping each method in a request function.
 * The resulting proxy object allows invoking methods of the original class, but their execution
 * is delegated to the request function. This is useful when executing methods across different
 * environments or threads, such as Web Workers or Node.js processes.
 *
 * @typeParam T - The type of the class to create a proxy for.
 * @param class_ - The class constructor to create a proxy for.
 * @param requestFn - A higher-order function that takes a method name and returns a function
 *                    with the same signature as the original method, which wraps the actual
 *                    invocation in a custom logic (e.g., sending a message to another thread).
 * @returns An object whose methods match those of the original class, but whose execution is
 *          delegated to the provided request function.
 */
export function createDispatchProxyFromFn<T>(
  class_: { new (...args: any[]): T },
  requestFn: (fn: string) => (...args: any[]) => Promise<any>,
): Proxify<T> {
  const proxy: any = class_.prototype instanceof EventEmitter ? new EventEmitter() : {};
  for (const fn of Object.getOwnPropertyNames(class_.prototype)) {
    if (fn === 'constructor') {
      continue;
    }
    proxy[fn] = requestFn(fn);
  }
  return proxy;
}

/**
 * Creates a proxy for the given class that transparently dispatches method calls over a transport client.
 * The proxy allows calling methods on remote instances of the class through the provided transport client
 * while maintaining the correct return types and handling promises. If the class inherits from EventEmitter,
 * it also handles event emissions through the transport client.
 *
 * @param class_ - The constructor function of the class to create the proxy for.
 * @param transportClient - The TransportClient instance used to handle communication between proxies.
 * @returns A proxified version of the given class with methods dispatched over the transport client.
 */
export function createDispatchProxy<T>(
  class_: { new (...args: any[]): T },
  transportClient: TransportClient<DispatchMsg>,
): Proxify<T> {
  // Create a proxy of class_ that passes along methods over our transportClient
  const proxy = createDispatchProxyFromFn(class_, (fn: string) => (...args: any[]) => {
    // Pass our proxied function name and arguments over our transport client
    const transfer: Transferable[] = args.reduce(
      (acc, a) => (isTransferDescriptor(a) ? [...acc, ...a.transferables] : acc),
      [] as Transferable[],
    );
    args = args.map(a => (isTransferDescriptor(a) ? a.send : a));
    return transportClient.request({ fn, args }, transfer);
  });
  if (proxy instanceof EventEmitter) {
    // Handle proxied 'emit' calls if our proxy object is an EventEmitter
    transportClient.on('event_msg', ({ fn, args }) => {
      if (fn === 'emit') {
        const [eventName, ...restArgs] = args;
        proxy.emit(eventName, ...restArgs);
      }
    });
  }
  return proxy;
}
