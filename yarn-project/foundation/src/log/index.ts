/** Structured log data to include with the message. */
export type LogData = Record<string, string | number | bigint | boolean>;

/** A callable logger instance. */
export type LogFn = (msg: string, data?: LogData) => void;

export * from './console.js';
export * from './debug.js';
export * from './logger.js';
export * from './log_history.js';
