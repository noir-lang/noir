import { jest } from '@jest/globals';

import { Mutex } from './index.js';
import { MutexDatabase } from './mutex_database.js';

/**
 * Sleep function for introducing a delay in the execution of code.
 * Returns a Promise that resolves after the specified number of milliseconds.
 *
 * @param ms - The number of milliseconds to pause the execution.
 * @returns A Promise that resolves after the specified delay.
 */
export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Type that transforms the properties of an object into jest.Mock instances,
 * allowing for easy mocking and testing of functions and methods.
 */
type Mockify<T> = {
  [P in keyof T]: jest.Mock;
};

describe('mutex', () => {
  let db: Mockify<MutexDatabase>;
  let mutex: Mutex;
  const mutexName = 'test-mutex';
  const timeout = 500;
  const tryLockInterval = 100;
  const pingInterval = 200;

  beforeEach(() => {
    db = {
      acquireLock: jest.fn().mockImplementation(() => false),
      extendLock: jest.fn().mockImplementation(() => {
        (db.acquireLock.mockResolvedValueOnce as any)(false);
      }),
      releaseLock: jest.fn().mockImplementation(() => {
        (db.acquireLock.mockResolvedValueOnce as any)(true);
      }),
    } as any;
    (db.acquireLock.mockResolvedValueOnce as any)(true);

    mutex = new Mutex(db as MutexDatabase, mutexName, timeout, tryLockInterval, pingInterval);
  });

  it('cannot lock if locked', async () => {
    const result: string[] = [];
    const fn1 = async (runAfterLocked: () => Promise<void>) => {
      await mutex.lock();
      const pm = runAfterLocked();
      await sleep(500);
      result.push('fn1');
      await mutex.unlock();
      return pm;
    };

    const fn2 = async () => {
      await mutex.lock();
      result.push('fn2');
      await mutex.unlock();
    };

    await fn1(fn2);
    expect(result).toEqual(['fn1', 'fn2']);
  });

  it('automatically extend the expiry time of the lock', async () => {
    await mutex.lock();
    await sleep(1000);
    await mutex.unlock();

    expect(db.extendLock).toHaveBeenCalledWith(mutexName, timeout);
  });
});
