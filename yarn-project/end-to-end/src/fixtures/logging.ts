import { onLog } from '@aztec/foundation/log';

import { mkdirpSync } from 'fs-extra';
import { dirname } from 'path';
import * as winston from 'winston';

const { format, transports } = winston;

let metricsLoggerSet = false;

/** Returns whether metrics logging should be enabled by default, checking env vars CI and BENCHMARK. */
export function isMetricsLoggingRequested() {
  return !!(process.env.CI || process.env.BENCHMARK);
}

/**
 * Configures an NDJSON logger to output entries to a local file that have an `eventName` associated.
 * Idempotent and automatically called by `setup` if CI or BENCHMARK env vars are set.
 */
export function setupMetricsLogger(filename: string) {
  if (metricsLoggerSet) return;
  mkdirpSync(dirname(filename));
  const logger = winston.createLogger({
    level: 'debug',
    format: format.combine(format.timestamp(), format.json()),
    transports: [new transports.File({ filename })],
  });
  onLog((level, namespace, message, data) => {
    if (data && data['eventName']) {
      logger.log({ ...data, level, namespace, message });
    }
  });
  metricsLoggerSet = true;
}
