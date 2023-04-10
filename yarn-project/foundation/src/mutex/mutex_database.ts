/**
 * Represents a mutual exclusion (mutex) database interface.
 * Provides functionality for acquiring, extending, and releasing locks on resources to ensure exclusive access and prevent conflicts in concurrent applications.
 */
export interface MutexDatabase {
  acquireLock(name: string, timeout: number): Promise<boolean>;
  extendLock(name: string, timeout: number): Promise<void>;
  releaseLock(name: string): Promise<void>;
}
