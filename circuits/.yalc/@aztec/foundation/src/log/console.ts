export type Logger = (...args: any[]) => void;

class ConsoleLogger {
  constructor(private prefix: string, private logger: (...args: any[]) => void = console.log) {}

  public log(...args: any[]) {
    this.logger(`${this.prefix}:`, ...args);
  }
}

export function createLogger(prefix: string): Logger {
  if (prefix) {
    const logger = new ConsoleLogger(prefix, console.log);
    return (...args: any[]) => logger.log(...args);
  }
  return console.log;
}
