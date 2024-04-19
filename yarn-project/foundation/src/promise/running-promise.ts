import { InterruptibleSleep } from '../sleep/index.js';

/**
 * RunningPromise is a utility class that helps manage the execution of an asynchronous function
 * at a specified polling interval. It allows starting, stopping, and checking the status of the
 * internally managed promise. The class also supports interrupting the polling process when stopped.
 */
export class RunningPromise {
  private running = false;
  private runningPromise = Promise.resolve();
  private interruptibleSleep = new InterruptibleSleep();

  constructor(private fn: () => Promise<void>, private pollingIntervalMS = 10000) {}

  /**
   * Starts the running promise.
   */
  public start() {
    this.running = true;

    const poll = async () => {
      while (this.running) {
        await this.fn();
        await this.interruptibleSleep.sleep(this.pollingIntervalMS);
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
    this.interruptibleSleep.interrupt();
    await this.runningPromise;
  }

  /**
   * Checks if the running promise is currently active.
   * @returns True if the promise is running.
   */
  public isRunning() {
    return this.running;
  }
}
