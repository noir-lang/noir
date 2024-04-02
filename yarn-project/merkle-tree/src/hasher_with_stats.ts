import { type Hasher } from '@aztec/types/interfaces';

import { createHistogram, performance } from 'perf_hooks';

/**
 * A helper class to track stats for a Hasher
 */
export class HasherWithStats implements Hasher {
  hashCount = 0;
  hashInputsCount = 0;
  hashHistogram = createHistogram();
  hashInputsHistogram = createHistogram();

  hash: Hasher['hash'];
  hashInputs: Hasher['hashInputs'];

  constructor(hasher: Hasher) {
    this.hash = performance.timerify(
      (lhs, rhs) => {
        this.hashCount++;
        return hasher.hash(lhs, rhs);
      },
      { histogram: this.hashHistogram },
    );
    this.hashInputs = performance.timerify(
      (inputs: Buffer[]) => {
        this.hashInputsCount++;
        return hasher.hashInputs(inputs);
      },
      { histogram: this.hashInputsHistogram },
    );
  }

  stats() {
    return {
      hashCount: this.hashCount,
      // timerify records in ns, convert to ms
      hashDuration: this.hashHistogram.mean / 1e6,
      hashInputsCount: this.hashInputsCount,
      hashInputsDuration: this.hashInputsHistogram.mean / 1e6,
    };
  }

  reset() {
    this.hashCount = 0;
    this.hashHistogram.reset();

    this.hashInputsCount = 0;
    this.hashInputsHistogram.reset();
  }
}
