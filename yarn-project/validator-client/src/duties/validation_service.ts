import { BlockAttestation, BlockProposal, type TxHash } from '@aztec/circuit-types';
import { type Header } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type ValidatorKeyStore } from '../key_store/interface.js';

export class ValidationService {
  constructor(private keyStore: ValidatorKeyStore) {}

  /**
   * Create a block proposal with the given header, archive, and transactions
   *
   * @param header - The block header
   * @param archive - The archive of the current block
   * @param txs - TxHash[] ordered list of transactions
   *
   * @returns A block proposal signing the above information (not the current implementation!!!)
   */
  async createBlockProposal(header: Header, archive: Fr, txs: TxHash[]): Promise<BlockProposal> {
    // Note: just signing the archive for now
    const archiveBuf = archive.toBuffer();
    const sig = await this.keyStore.sign(archiveBuf);

    return new BlockProposal(header, archive, txs, sig);
  }

  /**
   * Attest to the given block proposal constructed by the current sequencer
   *
   * @param proposal - The proposal to attest to
   * @returns attestation
   */
  async attestToProposal(proposal: BlockProposal): Promise<BlockAttestation> {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/7961): check that the current validator is correct

    const buf = proposal.archive.toBuffer();
    const sig = await this.keyStore.sign(buf);
    return new BlockAttestation(proposal.header, proposal.archive, sig);
  }
}
