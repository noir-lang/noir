import { Ecdsa, Grumpkin, Schnorr, Secp256k1 } from '@aztec/circuits.js/barretenberg';

/**
 * Currently supported curve implementations
 */
export enum CurveType {
  GRUMPKIN = 1,
  SECP256K1 = 2,
}

/**
 * Currently supported signer implementations
 */
export enum SignerType {
  SCHNORR = 1,
  ECDSA = 2,
}

/**
 * Factory method for creating the required curve implementation.
 */
export function createCurve(curveType: CurveType) {
  switch (curveType) {
    case CurveType.GRUMPKIN:
      return Grumpkin.new();
    case CurveType.SECP256K1:
      return Secp256k1.new();
  }
  throw new Error(`Unsupported curve type`);
}

/**
 * Factory method for creating the required signer implementation.
 */
export function createSigner(signerType: SignerType) {
  switch (signerType) {
    case SignerType.SCHNORR:
      return Schnorr.new();
    case SignerType.ECDSA:
      return Ecdsa.new();
  }
}
