import debug from 'debug';

let preLogHook: ((...args: any[]) => void) | undefined;
let postLogHook: ((...args: any[]) => void) | undefined;

/**
 *
 */
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
 *
 */
export function createDebugLogger(name: string): any {
  const logger = debug(name);
  return (...args: any[]) => theFunctionThroughWhichAllLogsPass(logger, ...args);
}

/**
 *
 */
export function setPreDebugLogHook(fn: (...args: any[]) => void) {
  preLogHook = fn;
}

/**
 *
 */
export function setPostDebugLogHook(fn: (...args: any[]) => void) {
  postLogHook = fn;
}

/**
 *
 */
export function enableLogs(str: string) {
  debug.enable(str);
}

/**
 *
 */
export function isLogEnabled(str: string) {
  return debug.enabled(str);
}
