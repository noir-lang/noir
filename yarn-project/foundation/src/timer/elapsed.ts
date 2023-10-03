import { Timer } from './timer.js';

/**
 * Measures the elapsed execution time of a function call or promise once it is awaited.
 * @param fn - Function or promise.
 * @returns A timer object.
 */
export async function elapsed<T>(fn: Promise<T> | (() => T | Promise<T>)): Promise<[Timer, T]> {
  const timer = new Timer();
  const result = await (typeof fn === 'function' ? fn() : fn);
  return [timer, result];
}
