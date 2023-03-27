export class RunningPromise {
  private running = false;
  private runningPromise = Promise.resolve();
  private interruptPromise = Promise.resolve();
  private interruptResolve = () => {};
  constructor(private fn: () => Promise<void>, private pollingInterval = 10000) {}

  /**
   * Starts the running promise
   */
  public start() {
    this.running = true;
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));

    const poll = async () => {
      while (this.running) {
        await this.fn();
        await this.interruptableSleep(this.pollingInterval);
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
