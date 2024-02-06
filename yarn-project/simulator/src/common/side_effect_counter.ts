/**
 * Keep track of the number of side effects across execution contexts.
 */
export class SideEffectCounter {
  constructor(private value = 0) {}

  count() {
    const value = this.value;
    this.value++;
    return value;
  }
}
