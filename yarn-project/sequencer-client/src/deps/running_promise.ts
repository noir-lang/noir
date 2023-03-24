// yarn-project/data-archiver/src/polling_rollup_emitter.ts
export class RunningPromise {
  private running = false;
  private runningPromise = Promise.resolve();
  private interruptPromise = Promise.resolve();
  private interruptResolve = () => {};

  private pollingInterval: number;
  private includeRunningTime: boolean;

  constructor(private fn: () => Promise<void>, opts?: { pollingInterval?: number; includeRunningTime?: boolean }) {
    this.pollingInterval = opts?.pollingInterval ?? 10_000;
    this.includeRunningTime = opts?.includeRunningTime ?? false;
  }

  /**
   * Starts the running promise.
   */
  public start() {
    this.running = true;
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));

    const poll = async () => {
      while (this.running) {
        const start = Date.now();
        await this.fn();
        const elapsed = Date.now() - start;
        const sleepTime = this.includeRunningTime ? Math.max(this.pollingInterval - elapsed, 0) : this.pollingInterval;
        await this.interruptableSleep(sleepTime);
      }
    };
    this.runningPromise = poll();
  }

  async stop(): Promise<void> {
    this.running = false;
    this.interruptResolve();
    await this.runningPromise;
  }

  private async interruptableSleep(timeInMs: number) {
    let timeout!: NodeJS.Timeout;
    const sleepPromise = new Promise(resolve => {
      timeout = setTimeout(resolve, timeInMs);
    });
    await Promise.race([sleepPromise, this.interruptPromise]);
    clearTimeout(timeout);
  }

  public isRunning() {
    return this.running;
  }
}
