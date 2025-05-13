export class Timer {
  private start: number;

  constructor() {
    this.start = performance ? performance.now() : Date.now();
  }

  /**
   * Returns the elapsed time in milliseconds since the Timer instance was created.
   * Provides a simple and convenient way to measure the time duration between two events
   * or monitor performance of specific code sections.
   *
   * @returns The elapsed time in milliseconds.
   */
  public ms() {
    return (performance ? performance.now() : Date.now()) - this.start;
  }

  /**
   * Returns the time elapsed since the Timer instance was created, in seconds.
   * The value is calculated by subtracting the initial start time from the current time
   * and dividing the result by 1000 to convert milliseconds to seconds.
   *
   * @returns The elapsed time in seconds.
   */
  public s() {
    return this.ms() / 1000;
  }
}
