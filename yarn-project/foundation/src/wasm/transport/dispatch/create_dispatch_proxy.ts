import { DispatchMsg } from './create_dispatch_fn.js';
import { TransportClient } from '../transport_client.js';
import { EventEmitter } from 'events';
import { isTransferDescriptor, TransferDescriptor } from '../interface/transferable.js';

/**
 * Filters out attributes from the Base type, retaining only functions.
 * This utility type iterates through the keys of the given Base type and
 * preserves only the properties that are functions, excluding non-function attributes.
 */
type FilterOutAttributes<Base> = {
  [Key in keyof Base]: Base[Key] extends (...args: any) => any ? Base[Key] : never;
};

/**
 * Takes a function type F and transforms it into a type where the returned value is wrapped in a Promise.
 * This type is useful when turning synchronous functions into asynchronous ones by wrapping their return values
 * in Promises, ensuring that they can be used with async/await or thenable handlers.
 */
type PromisifyFunction<F extends (...args: any) => any> = (...args: Parameters<F>) => Promise<ReturnType<F>>;

/**
 * A mapped type that takes a base object with function properties and returns a new object
 * with the same function keys but with their return values wrapped in a Promise.
 * If the original function already returns a Promise, it remains unchanged.
 */
type Promisify<Base extends { [key: string]: (...args: any) => any }> = {
  [Key in keyof Base]: ReturnType<Base[Key]> extends Promise<any> ? Base[Key] : PromisifyFunction<Base[Key]>;
};

/**
 * Unpack transfer types.
 */
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
 * type MakeFunctionTransferrable\<TFunction extends (...args: any) =\> any\> = (
 *   ...args: TransferTypes\<Parameters\<TFunction\>\>
 * ) =\> ReturnType<TFunction>;.
 */
type MakeFunctionTransferrable<TFunction extends (...args: any) => any> = (
  ...args: TFunction extends (...args: infer P) => any ? TransferTypes<P> : never
) => ReturnType<TFunction>;

/**
 * Transferrable type maps over the methods of a given object Base, transforming their parameters
 * to accept either the original parameter types or their respective TransferDescriptor versions,
 * in case they are Transferable. This allows for efficient transfer of complex data structures like
 * ArrayBuffer across worker-thread boundaries.
 */
type Transferrable<Base extends { [key: string]: (...args: any[]) => any }> = {
  [Key in keyof Base]: MakeFunctionTransferrable<Base[Key]>;
};

/**
 * Proxify type represents a mapped version of an object with its methods transformed into
 * asynchronous, transferable, and promisified versions. It is used for creating proxies that
 * can handle method calls remotely, transferring data efficiently, and ensuring all returned
 * values are wrapped in Promises.
 */
export type Proxify<T> = Promisify<Transferrable<FilterOutAttributes<T>>>;

/**
 * Creates a proxy object of class T that uses the provided request function to handle method calls.
 * This function generates a proxy with same methods as in class T's prototype, and each method call
 * on the proxy will be handled by the provided request function. The request function should return a
 * Promise that resolves to the result of the original method call.
 *
 * @typeparam T - The type of the class to create a proxy for.
 * @param class_ - The class to create a proxy for.
 * @param requestFn - A function that takes a method name and returns a function handling method calls.
 * @returns A proxy object of class T with methods handled by the provided request function.
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
 * Create a proxy object of our class T that uses transportClient.
 * @param class_ - Our class T.
 * @param transportClient - The transport infrastructure.
 * @returns A proxy over T.
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
