export class TimeoutTask<T> {
  private interruptPromise!: Promise<any>;
  private interrupt = () => {};
  private totalTime = 0;

  constructor(private fn: () => Promise<T>, private timeout = 0, fnName = '') {
    this.interruptPromise = new Promise<T>((_, reject) => {
      this.interrupt = () => reject(new Error(`Timeout${fnName ? ` running ${fnName}` : ''} after ${timeout}ms.`));
    });
  }

  public async exec() {
    const interruptTimeout = !this.timeout ? 0 : setTimeout(this.interrupt, this.timeout);
    try {
      const start = Date.now();
      const result = await Promise.race<T>([this.fn(), this.interruptPromise]);
      this.totalTime = Date.now() - start;
      return result;
    } finally {
      clearTimeout(interruptTimeout);
    }
  }

  public getInterruptPromise() {
    return this.interruptPromise;
  }

  public getTime() {
    return this.totalTime;
  }
}

export const executeTimeout = async <T>(fn: () => Promise<T>, timeout = 0, fnName = '') => {
  const task = new TimeoutTask(fn, timeout, fnName);
  return await task.exec();
};
