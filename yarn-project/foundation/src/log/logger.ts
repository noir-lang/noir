import debug from 'debug';

import { type LogData, type LogFn } from './log_fn.js';

const LogLevels = ['silent', 'error', 'warn', 'info', 'verbose', 'debug'] as const;
const DefaultLogLevel = process.env.NODE_ENV === 'test' ? ('silent' as const) : ('info' as const);

/**
 * A valid log severity level.
 */
export type LogLevel = (typeof LogLevels)[number];

const envLogLevel = process.env.LOG_LEVEL?.toLowerCase() as LogLevel;
export const currentLevel = LogLevels.includes(envLogLevel) ? envLogLevel : DefaultLogLevel;

const namespaces = process.env.DEBUG ?? 'aztec:*';
debug.enable(namespaces);

/** Log function that accepts an exception object */
type ErrorLogFn = (msg: string, err?: Error | unknown, data?: LogData) => void;

/**
 * Logger that supports multiple severity levels.
 */
export type Logger = { [K in LogLevel]: LogFn } & { /** Error log function */ error: ErrorLogFn };

/**
 * Logger that supports multiple severity levels and can be called directly to issue a debug statement.
 * Intended as a drop-in replacement for the debug module.
 */
export type DebugLogger = Logger;

/**
 * Creates a new DebugLogger for the current module, defaulting to the LOG_LEVEL env var.
 * If DEBUG="[module]" env is set, will enable debug logging if the module matches.
 * Uses npm debug for debug level and console.error for other levels.
 * @param name - Name of the module.
 * @returns A debug logger.
 */
export function createDebugLogger(name: string): DebugLogger {
  const debugLogger = debug(name);

  const logger = {
    silent: () => {},
    error: (msg: string, err?: unknown, data?: LogData) => logWithDebug(debugLogger, 'error', fmtErr(msg, err), data),
    warn: (msg: string, data?: LogData) => logWithDebug(debugLogger, 'warn', msg, data),
    info: (msg: string, data?: LogData) => logWithDebug(debugLogger, 'info', msg, data),
    verbose: (msg: string, data?: LogData) => logWithDebug(debugLogger, 'verbose', msg, data),
    debug: (msg: string, data?: LogData) => logWithDebug(debugLogger, 'debug', msg, data),
  };
  return Object.assign((msg: string, data?: LogData) => logWithDebug(debugLogger, 'debug', msg, data), logger);
}

/** A callback to capture all logs. */
export type LogHandler = (level: LogLevel, namespace: string, msg: string, data?: LogData) => void;

const logHandlers: LogHandler[] = [];

/**
 * Registers a callback for all logs, whether they are emitted in the current log level or not.
 * @param handler - Callback to be called on every log.
 */
export function onLog(handler: LogHandler) {
  logHandlers.push(handler);
}

/**
 * Logs args to npm debug if enabled or log level is debug, console.error otherwise.
 * @param debug - Instance of npm debug.
 * @param level - Intended log level.
 * @param args - Args to log.
 */
function logWithDebug(debug: debug.Debugger, level: LogLevel, msg: string, data?: LogData) {
  for (const handler of logHandlers) {
    handler(level, debug.namespace, msg, data);
  }

  msg = data ? `${msg} ${fmtLogData(data)}` : msg;
  if (debug.enabled && LogLevels.indexOf(level) <= LogLevels.indexOf(currentLevel)) {
    debug('[%s] %s', level.toUpperCase(), msg);
  }
}

/**
 * Concatenates a log message and an exception.
 * @param msg - Log message
 * @param err - Error to log
 * @returns A string with both the log message and the error message.
 */
function fmtErr(msg: string, err?: Error | unknown): string {
  const errStr = err && [(err as Error).name, (err as Error).message].filter(x => !!x).join(' ');
  return err ? `${msg}: ${errStr || err}` : msg;
}

/**
 * Formats structured log data as a string for console output.
 * @param data - Optional log data.
 */
function fmtLogData(data?: LogData): string {
  return Object.entries(data ?? {})
    .map(([key, value]) => `${key}=${typeof value === 'object' && 'toString' in value ? value.toString() : value}`)
    .join(' ');
}
