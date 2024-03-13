/**
 * Keep track of the number of side effects across execution contexts.
 */
export class SideEffectCounter {
  constructor(private value = 0) {}

  // TODO(alexg) remove this once public side effect counters is fully accounted for on Noir side
  current() {
    return this.value;
  }

  count() {
    const value = this.value;
    this.value++;
    return value;
  }
}
