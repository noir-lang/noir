/**
 * Represents a dispatch message interface.
 * Contains the target function name and its arguments to be executed in a dispatched manner.
 */
export interface DispatchMsg {
  /**
   * The name of the function to be called on the target object.
   */
  fn: string;
  /**
   * An array of arguments to be passed to the target function.
   */
  args: any[];
}

/**
 * Creates a dispatch function that calls a specified method on the target object returned by the targetFn.
 * The created dispatch function takes a single argument, an object containing the name of the function to be called (fn) and an array of arguments (args) to be passed to the function.
 * If the debug flag is enabled, it logs the dispatched call information to the console.
 *
 * @param targetFn - A function that returns the target object on which the method should be called.
 * @param debug - An optional console.error function for logging dispatched call information. Defaults to console.error.
 * @returns A dispatch function that takes a DispatchMsg object and calls the specified function on the target object with provided arguments.
 */
export function createDispatchFn(targetFn: () => any, debug = console.error) {
  return async ({ fn, args }: DispatchMsg) => {
    const target = targetFn();
    debug(`dispatching to ${target}: ${fn}`, args);
    return await target[fn](...args);
  };
}
