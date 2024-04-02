/* eslint-disable no-console */
import { type LogFn } from './log_fn.js';

/**
 * ConsoleLogger is a utility class that provides customizable console logging functionality.
 * It allows setting a custom prefix for log messages and an optional custom logger function,
 * which can be useful for controlling the format of the output or redirecting logs to a different destination.
 */
class ConsoleLogger {
  constructor(private prefix: string, private logger: (...args: any[]) => void = console.log) {}

  /**
   * Log messages with the specified prefix using the provided logger.
   * By default, it uses 'console.log' as the logger but can be overridden
   * during ConsoleLogger instantiation. This method allows for easy
   * organization and readability of log messages in the console.
   *
   * @param args - The data to be logged, any number of arguments can be passed to this function.
   */
  public log(...args: any[]) {
    this.logger(`${this.prefix}:`, ...args);
  }
}

/**
 * Creates a Logger function with an optional prefix for log messages.
 * If a prefix is provided, the created logger will prepend it to each log message.
 * If no prefix is provided, the default console.log will be returned.
 *
 * @param prefix - The optional string to prepend to each log message.
 * @returns A Logger function that accepts any number of arguments and logs them with the specified prefix.
 */
export function createConsoleLogger(prefix?: string): LogFn {
  if (prefix) {
    const logger = new ConsoleLogger(prefix, console.log);
    return (...args: any[]) => logger.log(...args);
  }
  return console.log;
}
