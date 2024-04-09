import { format } from 'util';

import { createDebugLogger } from '../../log/index.js';

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
 *
 * @param targetFn - A function that returns the target object containing the methods to be dispatched.
 * @param log - Optional logging function for debugging purposes.
 * @returns A dispatch function that accepts a DispatchMsg object and calls the target's method with provided arguments.
 */
export function createDispatchFn(targetFn: () => any, log = createDebugLogger('aztec:foundation:dispatch')) {
  return async ({ fn, args }: DispatchMsg) => {
    const target = targetFn();
    log.debug(format(`dispatching to ${target}: ${fn}`, args));
    return await target[fn](...args);
  };
}
