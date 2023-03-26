import { DispatchMsg } from './create_dispatch_fn.js';
import { TransportClient } from '../transport_client.js';
import { EventEmitter } from 'events';
import { isTransferDescriptor, TransferDescriptor } from '../interface/transferable.js';

type FilterOutAttributes<Base> = {
  [Key in keyof Base]: Base[Key] extends (...any: any) => any ? Base[Key] : never;
};

type PromisifyFunction<F extends (...any: any) => any> = (...args: Parameters<F>) => Promise<ReturnType<F>>;

type Promisify<Base extends { [key: string]: (...any: any) => any }> = {
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
type MakeFunctionTransferrable<TFunction extends (...args: any) => any> = (
  ...args: TFunction extends (...args: infer P) => any ? TransferTypes<P> : never
) => ReturnType<TFunction>;

type Transferrable<Base extends { [key: string]: (...any: any) => any }> = {
  [Key in keyof Base]: MakeFunctionTransferrable<Base[Key]>;
};

export type Proxify<T> = Promisify<Transferrable<FilterOutAttributes<T>>>;

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
