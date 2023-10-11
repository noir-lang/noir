import { Timer } from './timer.js';

/**
 * Measures the elapsed execution time of a function call or promise once it is awaited.
 * @param fn - Function or promise.
 * @returns The number of ms and the result.
 */
export async function elapsed<T>(fn: Promise<T> | (() => T | Promise<T>)): Promise<[number, T]> {
  const timer = new Timer();
  const result = await (typeof fn === 'function' ? fn() : fn);
  return [timer.ms(), result];
}

/**
 * Measures the elapsed execution time of a synchronous function call once it is awaited.
 * @param fn - Function.
 * @returns The number of ms and the result.
 */
export function elapsedSync<T>(fn: () => T): [number, T] {
  const timer = new Timer();
  const result = fn();
  return [timer.ms(), result];
}
