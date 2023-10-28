import * as fs from 'fs';
export * from './timer.js';

const bfd = (() => {
  const bfdStr = process.env.BENCHMARK_FD;
  const bfd = bfdStr ? parseInt(bfdStr) : -1;
  if (bfd >= 0 && !fs.fstatSync(bfd)) {
    throw new Error('fd is not open. Did you redirect in your shell?');
  }
  return bfd;
})();

export function writeBenchmark<T>(name: string, value: T, labels: Record<string, any> = {}) {
  if (bfd === -1) {
    return;
  }
  const data = {
    timestamp: new Date().toISOString(),
    name,
    type: typeof value,
    value,
    ...labels,
  };
  const jsonl = JSON.stringify(data) + '\n';
  fs.writeSync(bfd, jsonl);
}
