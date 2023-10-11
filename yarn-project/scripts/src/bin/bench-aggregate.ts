import { main } from '../benchmarks/aggregate.js';

void main().catch(err => {
  // eslint-disable-next-line no-console
  console.error(err.message);
  process.exit(1);
});
