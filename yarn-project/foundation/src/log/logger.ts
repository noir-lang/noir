import debug from 'debug';
import isNode from 'detect-node';
import { isatty } from 'tty';

import { LogFn } from './index.js';

const LogLevels = ['silent', 'fatal', 'error', 'warn', 'info', 'debug'] as const;
const DefaultLogLevel = 'info' as const;

/**
 * A valid log severity level.
 */
type LogLevel = (typeof LogLevels)[number];

const envLogLevel = process.env.LOG_LEVEL?.toLowerCase() as LogLevel;
const currentLevel = LogLevels.includes(envLogLevel) ? envLogLevel : DefaultLogLevel;

/**
 * Logger that supports multiple severity levels.
 */
export type Logger = { [K in LogLevel]: LogFn };

/**
 * Logger that supports multiple severity levels and can be called directly to issue a debug statement.
 * Intended as a drop-in replacement for the debug module.
 */
export type DebugLogger = LogFn & Logger;

/**
 * Creates a new DebugLogger for the current module, defaulting to the LOG_LEVEL env var.
 * If DEBUG="[module]" env is set, will enable debug logging if the module matches.
 * Uses npm debug for debug level and console.error for other levels.
 * @param name - Name of the module.
 * @returns A debug logger.
 */
export function createDebugLogger(name: string): DebugLogger {
  const debugLogger = debug(name);
  if (currentLevel === 'debug') debugLogger.enabled = true;

  const logger = {
    silent: () => {},
    fatal: (...args: any[]) => logWithDebug(debugLogger, 'fatal', args),
    error: (...args: any[]) => logWithDebug(debugLogger, 'error', args),
    warn: (...args: any[]) => logWithDebug(debugLogger, 'warn', args),
    info: (...args: any[]) => logWithDebug(debugLogger, 'info', args),
    debug: (...args: any[]) => logWithDebug(debugLogger, 'debug', args),
  };
  return Object.assign(debugLogger, logger);
}

/**
 * Logs args to npm debug if enabled or log level is debug, console.error otherwise.
 * @param debug - Instance of npm debug.
 * @param level - Intended log level.
 * @param args - Args to log.
 */
function logWithDebug(debug: debug.Debugger, level: LogLevel, args: any[]) {
  if (debug.enabled) {
    debug(args[0], ...args.slice(1));
  } else if (LogLevels.indexOf(level) <= LogLevels.indexOf(currentLevel) && process.env.NODE_ENV !== 'test') {
    printLog([getPrefix(debug, level), ...args]);
  }
}

/**
 * Returns a log prefix that emulates that of npm debug. Uses colors if in node and in a tty.
 * @param debugLogger - Instance of npm debug logger.
 * @param level - Intended log level (printed out if strictly above current log level).
 * @returns Log prefix.
 */
function getPrefix(debugLogger: debug.Debugger, level: LogLevel) {
  const levelLabel = currentLevel !== level ? ` ${level.toUpperCase}` : '';
  const prefix = `${debugLogger.namespace.replace(/^aztec:/, '')}${levelLabel}`;
  if (!isNode || !isatty(process.stderr.fd)) return prefix;
  const colorIndex = debug.selectColor(debugLogger.namespace) as number;
  const colorCode = '\u001B[3' + (colorIndex < 8 ? colorIndex : '8;5;' + colorIndex);
  return `  ${colorCode};1m${prefix}\u001B[0m`;
}

/**
 * Outputs to console error.
 * @param args - Args to log.
 */
function printLog(args: any[]) {
  // eslint-disable-next-line no-console
  console.error(...args);
}
