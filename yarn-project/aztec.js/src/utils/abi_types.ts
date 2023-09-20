import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';

/** Any type that can be converted into a field for a contract call. */
export type FieldLike = Fr | Buffer | bigint | number | { /** Converts to field */ toField: () => Fr };

/** Any type that can be converted into an EthereumAddress Aztec.nr struct. */
export type EthAddressLike = { /** Wrapped address */ address: FieldLike } | EthAddress;

/** Any type that can be converted into an EthereumAddress Aztec.nr struct. */
export type AztecAddressLike = { /** Wrapped address */ address: FieldLike } | AztecAddress;
