export type PromiseWithResolvers<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason?: any) => void;
};

/**
 * A polyfill for the Promise.withResolvers proposed API.
 * @see https://github.com/tc39/proposal-promise-with-resolvers
 * @returns A promise with resolvers.
 */
export function promiseWithResolvers<T>(): PromiseWithResolvers<T> {
  // use ! operator to avoid TS error
  let resolve!: (value: T) => void;
  let reject!: (reason?: any) => void;

  // the ES spec guarantees that the promise executor is called synchronously
  // so the resolve and reject functions will be defined
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });

  return {
    promise,
    resolve,
    reject,
  };
}
