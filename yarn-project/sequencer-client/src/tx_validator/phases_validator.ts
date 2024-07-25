import {
  type AllowedElement,
  type PublicExecutionRequest,
  PublicKernelType,
  Tx,
  type TxValidator,
} from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { AbstractPhaseManager, ContractsDataSourcePublicDB } from '@aztec/simulator';
import { type ContractDataSource } from '@aztec/types/contracts';

export class PhasesTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_phases');
  private contractDataSource: ContractsDataSourcePublicDB;

  constructor(contracts: ContractDataSource, private setupAllowList: AllowedElement[]) {
    this.contractDataSource = new ContractsDataSourcePublicDB(contracts);
  }

  async validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];

    for (const tx of txs) {
      // TODO(@spalladino): We add this just to handle public authwit-check calls during setup
      // which are needed for public FPC flows, but fail if the account contract hasnt been deployed yet,
      // which is what we're trying to do as part of the current txs.
      await this.contractDataSource.addNewContracts(tx);

      if (await this.#validateTx(tx)) {
        validTxs.push(tx);
      } else {
        invalidTxs.push(tx);
      }

      await this.contractDataSource.removeNewContracts(tx);
    }

    return Promise.resolve([validTxs, invalidTxs]);
  }

  async #validateTx(tx: Tx): Promise<boolean> {
    if (!tx.data.forPublic) {
      this.#log.debug(`Tx ${Tx.getHash(tx)} does not contain enqueued public functions. Skipping phases validation.`);
      return true;
    }

    const { [PublicKernelType.SETUP]: setupFns } = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(tx);

    for (const setupFn of setupFns) {
      if (!(await this.isOnAllowList(setupFn, this.setupAllowList))) {
        this.#log.warn(
          `Rejecting tx ${Tx.getHash(tx)} because it calls setup function not on allow list: ${
            setupFn.contractAddress
          }:${setupFn.callContext.functionSelector}`,
        );

        return false;
      }
    }

    return true;
  }

  async isOnAllowList(publicCall: PublicExecutionRequest, allowList: AllowedElement[]): Promise<boolean> {
    if (publicCall.isEmpty()) {
      return true;
    }

    const {
      contractAddress,
      callContext: { functionSelector },
    } = publicCall;

    // do these checks first since they don't require the contract class
    for (const entry of allowList) {
      if ('address' in entry && !('selector' in entry)) {
        if (contractAddress.equals(entry.address)) {
          return true;
        }
      }

      if ('address' in entry && 'selector' in entry) {
        if (contractAddress.equals(entry.address) && entry.selector.equals(functionSelector)) {
          return true;
        }
      }

      const contractClass = await this.contractDataSource.getContractInstance(contractAddress);

      if (!contractClass) {
        throw new Error(`Contract not found: ${publicCall.contractAddress.toString()}`);
      }

      if ('classId' in entry && !('selector' in entry)) {
        if (contractClass.contractClassId.equals(entry.classId)) {
          return true;
        }
      }

      if ('classId' in entry && 'selector' in entry) {
        if (
          contractClass.contractClassId.equals(entry.classId) &&
          (entry.selector === undefined || entry.selector.equals(functionSelector))
        ) {
          return true;
        }
      }
    }

    return false;
  }
}
