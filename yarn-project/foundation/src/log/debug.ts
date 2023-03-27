import debug from 'debug';

let preLogHook: ((...args: any[]) => void) | undefined;
let postLogHook: ((...args: any[]) => void) | undefined;

function theFunctionThroughWhichAllLogsPass(logger: any, ...args: any[]) {
  if (!debug.enabled(logger.namespace)) {
    return;
  }
  if (preLogHook) {
    preLogHook(logger.namespace, ...args);
  }
  logger(...args);
  if (postLogHook) {
    postLogHook(logger.namespace, ...args);
  }
}

/**
 * A debug logger.
 */
export type DebugLogger = (...args: any[]) => void;

/**
 * Return a logger, meant to be silent by default and verbose during debugging.
 * @param name - The module name of the logger.
 * @returns A callable log function.
 */
export function createDebugLogger(name: string): DebugLogger {
  const logger = debug(name);
  return (...args: any[]) => theFunctionThroughWhichAllLogsPass(logger, ...args);
}

export function setPreDebugLogHook(fn: (...args: any[]) => void) {
  preLogHook = fn;
}

export function setPostDebugLogHook(fn: (...args: any[]) => void) {
  postLogHook = fn;
}

export function enableLogs(str: string) {
  debug.enable(str);
}

export function isLogEnabled(str: string) {
  return debug.enabled(str);
}
