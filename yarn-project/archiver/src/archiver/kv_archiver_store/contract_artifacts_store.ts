import { type AztecAddress } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';
import { contractArtifactFromBuffer, contractArtifactToBuffer } from '@aztec/types/abi';

export class ContractArtifactsStore {
  #contractArtifacts: AztecMap<string, Buffer>;

  constructor(db: AztecKVStore) {
    this.#contractArtifacts = db.openMap('archiver_contract_artifacts');
  }

  addContractArtifact(address: AztecAddress, contractArtifact: ContractArtifact): Promise<void> {
    return this.#contractArtifacts.set(address.toString(), contractArtifactToBuffer(contractArtifact));
  }

  getContractArtifact(address: AztecAddress): ContractArtifact | undefined {
    const contractArtifact = this.#contractArtifacts.get(address.toString());
    // TODO(@spalladino): AztecMap lies and returns Uint8Arrays instead of Buffers, hence the extra Buffer.from.
    return contractArtifact && contractArtifactFromBuffer(Buffer.from(contractArtifact));
  }
}
