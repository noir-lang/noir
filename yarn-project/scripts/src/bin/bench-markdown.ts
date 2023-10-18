import { main } from '../benchmarks/markdown.js';

try {
  void main();
} catch (err: any) {
  // eslint-disable-next-line no-console
  console.error(err);
  process.exit(1);
}
