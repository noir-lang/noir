export class InterruptError extends Error {}

export class InterruptableSleep {
  private interruptResolve: (shouldThrow: boolean) => void = () => {};
  private interruptPromise = new Promise<boolean>(resolve => (this.interruptResolve = resolve));
  private timeouts: NodeJS.Timeout[] = [];

  public async sleep(ms: number) {
    let timeout!: NodeJS.Timeout;
    const promise = new Promise<boolean>(resolve => (timeout = setTimeout(() => resolve(false), ms)));
    this.timeouts.push(timeout);
    const shouldThrow = await Promise.race([promise, this.interruptPromise]);
    clearTimeout(timeout);
    this.timeouts.splice(this.timeouts.indexOf(timeout), 1);
    if (shouldThrow) {
      throw new InterruptError('Interrupted.');
    }
  }

  public interrupt(sleepShouldThrow = false) {
    this.interruptResolve(sleepShouldThrow);
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));
  }
}

export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
