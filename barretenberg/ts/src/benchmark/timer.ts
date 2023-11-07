/**
 * Timer class to measure time intervals in milliseconds and seconds.
 * Upon instantiation, it stores the current timestamp as the starting point.
 * The 'ms()' method returns the elapsed time in milliseconds,
 * while the 's()' method returns the elapsed time in seconds.
 *
 * @example
 * const timer = new Timer();
 * setTimeout(() =\> \{
 *   console.log(`Elapsed time: ${timer.ms()} ms`);
 * \}, 1000);
 */
export class Timer {
  private start: number;

  constructor() {
    this.start = performance.now();
  }

  public us() {
    return this.ms() * 1000;
  }

  /**
   * Returns the elapsed time in milliseconds since the Timer instance was created.
   * Provides a simple and convenient way to measure the time duration between two events
   * or monitor performance of specific code sections.
   *
   * @returns The elapsed time in milliseconds.
   */
  public ms() {
    return performance.now() - this.start;
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
