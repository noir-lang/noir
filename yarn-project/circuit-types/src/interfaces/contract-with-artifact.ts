import { ContractArtifact } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { ContractInstanceWithAddress } from '@aztec/types/contracts';

/**
 * Container for contract instance and artifact or class id.
 */
export type ContractWithArtifact = (
  | {
      /** The artifact of the contract. */
      artifact: ContractArtifact;
    }
  | {
      /** The class id of the contract artifact. */
      contractClassId: Fr;
    }
) & {
  /** The contract instance. */
  instance: ContractInstanceWithAddress;
};
