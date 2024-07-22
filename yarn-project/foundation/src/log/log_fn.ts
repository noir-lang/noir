/** Structured log data to include with the message. */
export type LogData = Record<string, string | number | bigint | boolean | { toString(): string } | undefined>;

/** A callable logger instance. */
export type LogFn = (msg: string, data?: LogData) => void;
