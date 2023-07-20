import { jest } from '@jest/globals';

import { createDebugOnlyLogger, enableLogs } from './debug.js';
import { LogHistory } from './log_history.js';

jest.useFakeTimers({ doNotFake: ['performance'] });

describe('log history', () => {
  let debug: (...any: any) => void;
  let logHistory: LogHistory;
  const timestemp = new Date().toISOString();
  const name = 'test:a';

  beforeEach(() => {
    debug = createDebugOnlyLogger(name);
    enableLogs(name);
    logHistory = new LogHistory();
  });

  it('keeps debug logs', () => {
    logHistory.enable();
    expect(logHistory.getLogs()).toEqual([]);
    debug('0');
    debug('1', 2);
    debug('2', { key: ['value'] }, Buffer.alloc(2));
    expect(logHistory.getLogs()).toEqual([
      [timestemp, name, '0'],
      [timestemp, name, '1', 2],
      [timestemp, name, '2', { key: ['value'] }, Buffer.alloc(2)],
    ]);
  });

  it('does not keep logs if not enabled', () => {
    debug('0');
    debug('1', 2);
    expect(logHistory.getLogs()).toEqual([]);
  });

  it('returns last n logs', () => {
    logHistory.enable();
    expect(logHistory.getLogs()).toEqual([]);
    debug('0');
    debug('1');
    debug('2');
    debug('3');
    debug('4');
    expect(logHistory.getLogs(2)).toEqual([
      [timestemp, name, '3'],
      [timestemp, name, '4'],
    ]);
  });

  it('only keeps logs with enabled namespace', () => {
    logHistory.enable();
    const name2 = 'test:b';
    const debug2 = createDebugOnlyLogger(name2);
    debug('0');
    debug2('zero');
    expect(logHistory.getLogs()).toEqual([[timestemp, name, '0']]);

    enableLogs(`${name},${name2}`);
    debug('1', 2);
    debug2('one', 3);
    expect(logHistory.getLogs()).toEqual([
      [timestemp, name, '0'],
      [timestemp, name, '1', 2],
      [timestemp, name2, 'one', 3],
    ]);
  });

  it('clears all logs', () => {
    logHistory.enable();
    debug('0');
    debug('1');
    debug('2');
    logHistory.clear();
    expect(logHistory.getLogs()).toEqual([]);
  });

  it('clears first n logs', () => {
    logHistory.enable();
    debug('0');
    debug('1');
    debug('2');
    logHistory.clear(2);
    expect(logHistory.getLogs()).toEqual([[timestemp, name, '2']]);
  });
});
