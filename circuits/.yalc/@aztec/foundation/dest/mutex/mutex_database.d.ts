export interface MutexDatabase {
    acquireLock(name: string, timeout: number): Promise<boolean>;
    extendLock(name: string, timeout: number): Promise<void>;
    releaseLock(name: string): Promise<void>;
}
//# sourceMappingURL=mutex_database.d.ts.map