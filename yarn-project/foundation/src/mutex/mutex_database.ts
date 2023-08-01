/**
 * Represents a mutual exclusion (mutex) database interface.
 * Provides functionality for acquiring, extending, and releasing locks on resources to ensure exclusive access and prevent conflicts in concurrent applications.
 */
export interface MutexDatabase {
  // eslint-disable-next-line jsdoc/require-jsdoc
  acquireLock(name: string, timeout: number): Promise<boolean>;
  // eslint-disable-next-line jsdoc/require-jsdoc
  extendLock(name: string, timeout: number): Promise<void>;
  // eslint-disable-next-line jsdoc/require-jsdoc
  releaseLock(name: string): Promise<void>;
}
