import { type BlockAttestation } from '@aztec/circuit-types';

/**
 * An Attestation Pool contains attestations collected by a validator
 *
 * Attestations that are observed via the p2p network are stored for requests
 * from the validator to produce a block, or to serve to other peers.
 */
export interface AttestationPool {
  /**
   * AddAttestation
   *
   * @param attestations - Attestations to add into the pool
   */
  addAttestations(attestations: BlockAttestation[]): Promise<void>;

  /**
   * DeleteAttestation
   *
   * @param attestations - Attestations to remove from the pool
   */
  deleteAttestations(attestations: BlockAttestation[]): Promise<void>;

  /**
   * Delete Attestations for slot
   *
   * Removes all attestations associated with a slot
   *
   * @param slot - The slot to delete.
   */
  deleteAttestationsForSlot(slot: bigint): Promise<void>;

  /**
   * Get Attestations for slot
   *
   * Retrieve all of the attestations observed pertaining to a given slot
   *
   * @param slot - The slot to query
   * @return BlockAttestations
   */
  getAttestationsForSlot(slot: bigint): Promise<BlockAttestation[]>;
}
