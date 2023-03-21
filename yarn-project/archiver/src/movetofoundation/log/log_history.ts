import { setPreDebugLogHook } from './debug.js';

export class LogHistory {
  private logs: any[][] = [];

  public enable() {
    setPreDebugLogHook((...args: any[]) => {
      this.logs.push([new Date().toISOString(), ...args]);
    });
  }

  public getLogs(last = 0) {
    return last ? this.logs.slice(-last) : this.logs;
  }

  public clear(count = this.logs.length) {
    this.logs = this.logs.slice(count);
  }
}

export const logHistory = new LogHistory();
