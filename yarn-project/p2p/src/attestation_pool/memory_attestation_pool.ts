import { type BlockAttestation } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';

import { type AttestationPool } from './attestation_pool.js';

export class InMemoryAttestationPool implements AttestationPool {
  private attestations: Map</*slot=*/ bigint, Map</*address=*/ string, BlockAttestation>>;

  constructor(private log = createDebugLogger('aztec:attestation_pool')) {
    this.attestations = new Map();
  }

  public getAttestationsForSlot(slot: bigint): Promise<BlockAttestation[]> {
    const slotAttestationMap = this.attestations.get(slot);
    if (slotAttestationMap) {
      return Promise.resolve(Array.from(slotAttestationMap.values()));
    } else {
      return Promise.resolve([]);
    }
  }

  public async addAttestations(attestations: BlockAttestation[]): Promise<void> {
    for (const attestation of attestations) {
      // Perf: order and group by slot before insertion
      const slotNumber = attestation.header.globalVariables.slotNumber;

      const address = await attestation.getSender();

      const slotAttestationMap = getSlotOrDefault(this.attestations, slotNumber.toBigInt());
      slotAttestationMap.set(address.toString(), attestation);

      this.log.verbose(`Added attestation for slot ${slotNumber} from ${address}`);
    }
  }

  public deleteAttestationsForSlot(slot: bigint): Promise<void> {
    // TODO(md): check if this will free the memory of the inner hash map
    this.attestations.delete(slot);
    this.log.verbose(`Removed attestation for slot ${slot}`);
    return Promise.resolve();
  }

  public async deleteAttestations(attestations: BlockAttestation[]): Promise<void> {
    for (const attestation of attestations) {
      const slotNumber = attestation.header.globalVariables.slotNumber;
      const slotAttestationMap = this.attestations.get(slotNumber.toBigInt());
      if (slotAttestationMap) {
        const address = await attestation.getSender();
        slotAttestationMap.delete(address.toString());
        this.log.verbose(`Deleted attestation for slot ${slotNumber} from ${address}`);
      }
    }
    return Promise.resolve();
  }
}

/**
 * Get Slot or Default
 *
 * Fetch the slot mapping, if it does not exist, then create a mapping and return it
 */
function getSlotOrDefault(
  map: Map<bigint, Map<string, BlockAttestation>>,
  slot: bigint,
): Map<string, BlockAttestation> {
  if (!map.has(slot)) {
    map.set(slot, new Map<string, BlockAttestation>());
  }
  return map.get(slot)!;
}
