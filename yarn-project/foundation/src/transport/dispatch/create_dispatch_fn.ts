/**
 * Represents a message object for dispatching function calls.
 * Contains the function name ('fn') and an array of arguments ('args') required to call the target method.
 */
export interface DispatchMsg {
  /**
   * Name of the target method to be called.
   */
  fn: string;
  /**
   * An array of arguments to be passed to the target method.
   */
  args: any[];
}

/**
 * Creates a dispatch function that calls the target's specified method with provided arguments.
 * The created dispatch function takes a DispatchMsg object as input, which contains the name of
 * the method to be called ('fn') and an array of arguments to be passed to the method ('args').
 * An optional 'debug' parameter can be passed, which defaults to console.error, to log dispatched messages.
 *
 * @param targetFn - A function that returns the target object containing the methods to be dispatched.
 * @param debug - Optional logging function for debugging purposes (defaults to console.error).
 * @returns A dispatch function that accepts a DispatchMsg object and calls the target's method with provided arguments.
 */
export function createDispatchFn(targetFn: () => any, debug = console.error) {
  return async ({ fn, args }: DispatchMsg) => {
    const target = targetFn();
    debug(`dispatching to ${target}: ${fn}`, args);
    return await target[fn](...args);
  };
}
