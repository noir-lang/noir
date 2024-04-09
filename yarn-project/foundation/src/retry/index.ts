import { createDebugLogger } from '../log/index.js';
import { sleep } from '../sleep/index.js';
import { Timer } from '../timer/index.js';

/** An error that indicates that the operation should not be retried. */
export class NoRetryError extends Error {}

/**
 * Generates a backoff sequence for retrying operations with an increasing delay.
 * The backoff sequence follows this pattern: 1, 1, 1, 2, 4, 8, 16, 32, 64, ...
 * This generator can be used in combination with the `retry` function to perform
 * retries with exponential backoff and capped at 64 seconds between attempts.
 *
 * @returns A generator that yields the next backoff value in seconds as an integer.
 */
export function* backoffGenerator() {
  const v = [1, 1, 1, 2, 4, 8, 16, 32, 64];
  let i = 0;
  while (true) {
    yield v[Math.min(i++, v.length - 1)];
  }
}

/**
 * Generates a backoff sequence based on the array of retry intervals to use with the `retry` function.
 * @param retries - Intervals to retry (in seconds).
 * @returns A generator sequence.
 */
export function* makeBackoff(retries: number[]) {
  for (const retry of retries) {
    yield retry;
  }
}

/**
 * Retry a given asynchronous function with a specific backoff strategy, until it succeeds or backoff generator ends.
 * It logs the error and retry interval in case an error is caught. The function can be named for better log output.
 *
 * @param fn - The asynchronous function to be retried.
 * @param name - The optional name of the operation, used for logging purposes.
 * @param backoff - The optional backoff generator providing the intervals in seconds between retries. Defaults to a predefined series.
 * @param log - Logger to use for logging.
 * @param failSilently - Do not log errors while retrying.
 * @returns A Promise that resolves with the successful result of the provided function, or rejects if backoff generator ends.
 * @throws If `NoRetryError` is thrown by the `fn`, it is rethrown.
 */
export async function retry<Result>(
  fn: () => Promise<Result>,
  name = 'Operation',
  backoff = backoffGenerator(),
  log = createDebugLogger('aztec:foundation:retry'),
  failSilently = false,
) {
  while (true) {
    try {
      return await fn();
    } catch (err: any) {
      if (err instanceof NoRetryError) {
        // A special error that indicates that the operation should not be retried. Rethrow it.
        throw err;
      }
      const s = backoff.next().value;
      if (s === undefined) {
        throw err;
      }
      log.verbose(`${name} failed. Will retry in ${s}s...`);
      !failSilently && log.error(err);
      await sleep(s * 1000);
      continue;
    }
  }
}

/**
 * Retry an asynchronous function until it returns a truthy value or the specified timeout is exceeded.
 * The function is retried periodically with a fixed interval between attempts. The operation can be named for better error messages.
 * Will never timeout if the value is 0.
 *
 * @param fn - The asynchronous function to be retried, which should return a truthy value upon success or undefined otherwise.
 * @param name - The optional name of the operation, used for generating timeout error message.
 * @param timeout - The optional maximum time, in seconds, to keep retrying before throwing a timeout error. Defaults to 0 (never timeout).
 * @param interval - The optional interval, in seconds, between retry attempts. Defaults to 1 second.
 * @returns A Promise that resolves with the successful (truthy) result of the provided function, or rejects if timeout is exceeded.
 */
export async function retryUntil<T>(fn: () => Promise<T | undefined>, name = '', timeout = 0, interval = 1) {
  const timer = new Timer();
  while (true) {
    const result = await fn();
    if (result) {
      return result;
    }

    await sleep(interval * 1000);

    if (timeout && timer.s() > timeout) {
      throw new Error(name ? `Timeout awaiting ${name}` : 'Timeout');
    }
  }
}
