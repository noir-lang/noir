import { onLog, setLevel } from '@aztec/foundation/log';

import * as path from 'path';
import * as process from 'process';
import * as winston from 'winston';
import DailyRotateFile from 'winston-daily-rotate-file';

const { format } = winston;
const CURRENT_LOG_FILE_NAME = 'aztec.debug.log';
const LOG_DIR = 'log';

/** Creates a winston logger that logs everything to a local rotating file */
function createWinstonLocalFileLogger() {
  // See https://www.npmjs.com/package/winston-daily-rotate-file#user-content-options
  const transport: DailyRotateFile = new DailyRotateFile({
    filename: 'aztec-%DATE%.debug.log',
    dirname: LOG_DIR,
    datePattern: 'YYYY-MM-DD',
    zippedArchive: true,
    maxSize: '30m',
    maxFiles: '5',
    createSymlink: true,
    symlinkName: CURRENT_LOG_FILE_NAME,
  });

  return winston.createLogger({
    level: 'debug',
    transports: [transport],
    format: format.combine(format.timestamp(), format.json()),
  });
}

/** Creates a winston logger that logs everything to stdout in json format */
function createWinstonJsonStdoutLogger() {
  return winston.createLogger({
    level: process.env.LOG_LEVEL ?? 'info',
    transports: [new winston.transports.Console({ format: format.combine(format.timestamp(), format.json()) })],
  });
}

/**
 * Hooks to all log statements and outputs them to a local rotating file.
 * @returns Output log name.
 */
export function setupFileDebugLog() {
  const logger = createWinstonLocalFileLogger();
  onLog((level, module, message, data) => {
    logger.log({ ...data, level, module, message });
  });
  const workdir = process.env.HOST_WORKDIR ?? process.cwd();
  return path.join(workdir, LOG_DIR, CURRENT_LOG_FILE_NAME);
}

/**
 * Silences the foundation stdout logger and funnels all logs through a winston JSON logger.
 */
export function setupConsoleJsonLog() {
  const logger = createWinstonJsonStdoutLogger();
  setLevel('silent');
  onLog((level, module, message, data) => {
    logger.log({ ...data, level, module, message });
  });
}
