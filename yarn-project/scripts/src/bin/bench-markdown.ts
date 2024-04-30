import { main } from '../benchmarks/markdown.js';
import { getPrNumber } from '../utils/pr-number.js';

try {
  void main(getPrNumber());
} catch (err: any) {
  // eslint-disable-next-line no-console
  console.error(err);
  process.exit(1);
}
