import { Fr } from '@aztec/circuits.js';

/** Any type that can be converted into a field for a contract call. */
export type FieldLike = Fr | Buffer | bigint | number | { /** Converts to field */ toField: () => Fr };
