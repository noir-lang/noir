import { sleep } from '../sleep/index.js';
import { Timer } from '../timer/index.js';

/**
 *
 */
export function* backoffGenerator() {
  const v = [1, 1, 1, 2, 4, 8, 16, 32, 64];
  let i = 0;
  while (true) {
    yield v[Math.min(i++, v.length - 1)];
  }
}

/**
 *
 */
export async function retry<Result>(fn: () => Promise<Result>, name = 'Operation', backoff = backoffGenerator()) {
  while (true) {
    try {
      return await fn();
    } catch (err: any) {
      const s = backoff.next().value;
      if (s === undefined) {
        throw err;
      }
      console.log(`${name} failed. Will retry in ${s}s...`);
      console.log(err);
      await sleep(s * 1000);
      continue;
    }
  }
}

// Call `fn` repeatedly until it returns true or timeout.
// Both `interval` and `timeout` are seconds.
// Will never timeout if the value is 0.
/**
 *
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
