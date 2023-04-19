// eslint-disable-next-line jsdoc/require-jsdoc
export class InterruptError extends Error {}

/**
 * The InterruptableSleep class provides an enhanced sleep functionality that can be interrupted before the specified duration has elapsed.
 * It allows you to create sleep instances with specified durations, which can be interrupted by calling the 'interrupt' method on the instance.
 * In case of interruption, it can be configured to throw an 'InterruptError' or continue without throwing any error.
 * This is useful in scenarios where you want to break out of a sleep state based on external conditions or events.
 */
export class InterruptableSleep {
  private interruptResolve: (shouldThrow: boolean) => void = () => {};
  private interruptPromise = new Promise<boolean>(resolve => (this.interruptResolve = resolve));
  private timeouts: NodeJS.Timeout[] = [];

  // eslint-disable-next-line jsdoc/require-jsdoc
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

  // eslint-disable-next-line jsdoc/require-jsdoc
  public interrupt(sleepShouldThrow = false) {
    this.interruptResolve(sleepShouldThrow);
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));
  }
}

// eslint-disable-next-line jsdoc/require-jsdoc
export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
