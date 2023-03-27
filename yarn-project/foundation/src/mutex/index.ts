import { MutexDatabase } from './mutex_database.js';

export * from './mutex_database.js';

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

  public async unlock() {
    clearTimeout(this.pingTimeout);
    this.id++;
    await this.db.releaseLock(this.name);
  }

  private async ping(id: number) {
    if (id !== this.id) {
      return;
    }

    await this.db.extendLock(this.name, this.timeout);
    this.pingTimeout = setTimeout(() => this.ping(id), this.pingInterval);
  }
}
