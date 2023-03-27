import { jest } from '@jest/globals';
import { Mutex } from './index.js';
import { MutexDatabase } from './mutex_database.js';

export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

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
