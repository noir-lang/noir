/**
 * RunningPromise is a utility class that helps manage the execution of an asynchronous function
 * at a specified polling interval. It allows starting, stopping, and checking the status of the
 * internally managed promise. The class also supports interrupting the polling process when stopped.
 */
export class RunningPromise {
  private running = false;
  private runningPromise = Promise.resolve();
  private interruptPromise = Promise.resolve();
  private interruptResolve = () => {};
  constructor(private fn: () => Promise<void>, private pollingInterval = 10000) {}

  /**
   * Starts the running promise.
   */
  public start() {
    this.running = true;
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));

    const poll = async () => {
      while (this.running) {
        await this.fn();
        await this.interruptibleSleep(this.pollingInterval);
      }
    };
    this.runningPromise = poll();
  }

  /**
   * Stops the running promise, resolves any pending interruptible sleep,
   * and waits for the currently executing function to complete.
   */
  async stop(): Promise<void> {
    this.running = false;
    this.interruptResolve();
    await this.runningPromise;
  }

  /**
   * A sleep function that can be interrupted before the specified time.
   * The sleep duration is determined by 'timeInMs', and it can be terminated early if the 'interruptPromise' is resolved.
   * @param timeInMs - The time in milliseconds.
   */
  private async interruptibleSleep(timeInMs: number) {
    let timeout!: NodeJS.Timeout;
    const sleepPromise = new Promise(resolve => {
      timeout = setTimeout(resolve, timeInMs);
    });
    await Promise.race([sleepPromise, this.interruptPromise]);
    clearTimeout(timeout);
  }

  /**
   * Checks if the running promise is currently active.
   * @returns True if the promise is running.
   */
  public isRunning() {
    return this.running;
  }
}
