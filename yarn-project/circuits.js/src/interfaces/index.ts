import type { Fr } from '@aztec/foundation/fields';

export interface IsEmpty {
  isEmpty: () => boolean;
}

export interface Ordered {
  counter: number;
}

export interface Positioned {
  position: Fr;
}
