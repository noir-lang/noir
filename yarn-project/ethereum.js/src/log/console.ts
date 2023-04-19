// eslint-disable-next-line jsdoc/require-jsdoc
export type Logger = (...args: any[]) => void;

// eslint-disable-next-line jsdoc/require-jsdoc
class ConsoleLogger {
  constructor(private prefix: string, private logger: (...args: any[]) => void = console.log) {}

  // eslint-disable-next-line jsdoc/require-jsdoc
  public log(...args: any[]) {
    this.logger(`${this.prefix}:`, ...args);
  }
}

// eslint-disable-next-line jsdoc/require-jsdoc
export function createLogger(prefix: string): Logger {
  if (prefix) {
    const logger = new ConsoleLogger(prefix, console.log);
    return (...args: any[]) => logger.log(...args);
  }
  return console.log;
}
