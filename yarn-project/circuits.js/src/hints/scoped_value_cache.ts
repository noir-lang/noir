import type { AztecAddress } from '@aztec/foundation/aztec-address';
import type { Fr } from '@aztec/foundation/fields';

export class ScopedValueCache<T extends { value: Fr; contractAddress: AztecAddress }> {
  private cache: Map<bigint, T[]> = new Map();
  constructor(items: T[]) {
    items.forEach(item => {
      const value = item.value.toBigInt();
      const arr = this.cache.get(value) ?? [];
      arr.push(item);
      this.cache.set(value, arr);
    });
  }

  public get(matcher: { value: Fr; contractAddress: AztecAddress }): T[] {
    return this.cache.get(matcher.value.toBigInt()) ?? [];
  }
}
