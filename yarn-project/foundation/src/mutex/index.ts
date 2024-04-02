import { type MutexDatabase } from './mutex_database.js';

export * from './mutex_database.js';

/**
 * Mutex class provides a mutual exclusion mechanism for critical sections of code using a named lock.
 * The lock is acquired and released via the `lock` and `unlock` methods. Locks can be optionally pinged
 * to keep them alive when they are held for longer durations, avoiding unintended release due to timeouts.
 *
 * The underlying lock state is managed in a MutexDatabase instance which can be shared across multiple Mutex instances.
 * This allows for synchronization between different parts of an application or even across different instances of an application.
 *
 * @example
 * const mutex = new Mutex(mutexDatabase, 'myLock');
 * await mutex.lock();
 * // Critical section here
 * await mutex.unlock();
 */
export class Mutex {
  private id = 0;
  private pingTimeout!: NodeJS.Timeout;

  constructor(
    private readonly db: MutexDatabase,
    private readonly name: string,
    private readonly timeout = 5000,
    private readonly tryLockInterval = 2000,
    private readonly pingInterval = 2000,
  ) {}

  /**
   * Acquire a lock on the mutex. If 'untilAcquired' is true, the method will keep trying to acquire the lock until it
   * successfully acquires it. If 'untilAcquired' is false, the method will try to acquire the lock once and return
   * immediately with a boolean indicating if the lock has been acquired or not.
   *
   * @param untilAcquired - Optional parameter, set to true by default. If true, the method will keep trying to acquire the lock until success. If false, the method will try only once and return a boolean value.
   * @returns A Promise that resolves to true if the lock has been acquired, or false when 'untilAcquired' is false and the lock could not be immediately acquired.
   */
  public async lock(untilAcquired = true) {
    while (true) {
      if (await this.db.acquireLock(this.name, this.timeout)) {
        const id = this.id;
        this.pingTimeout = setTimeout(() => this.ping(id), this.pingInterval);
        return true;
      }

      if (!untilAcquired) {
        return false;
      }
      await new Promise(resolve => setTimeout(resolve, this.tryLockInterval));
    }
  }

  /**
   * Unlocks the mutex, allowing other instances to acquire the lock.
   * This method also clears the internal ping timeout and increments the internal ID
   * to ensure stale pings do not extend the lock after it has been released.
   *
   * @returns A promise that resolves once the lock has been released in the database.
   */
  public async unlock() {
    clearTimeout(this.pingTimeout);
    this.id++;
    await this.db.releaseLock(this.name);
  }

  /**
   * Periodically extends the lock's lifetime by updating the database record with a new expiration time.
   * This method is called recursively using setTimeout. If the id passed to the ping method does not match
   * the current lock instance's id, it means the lock has been released or acquired by another instance
   * and the ping should not proceed further.
   *
   * @param id - The id of the current lock instance.
   */
  private async ping(id: number) {
    if (id !== this.id) {
      return;
    }

    await this.db.extendLock(this.name, this.timeout);
    this.pingTimeout = setTimeout(() => this.ping(id), this.pingInterval);
  }
}
