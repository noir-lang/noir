/**
 * A callable logger instance.
 */
export type LogFn = (...args: any[]) => void;

export * from './console.js';
export * from './debug.js';
export * from './logger.js';
export * from './log_history.js';
