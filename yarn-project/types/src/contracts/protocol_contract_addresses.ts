import { type AztecAddress } from '@aztec/foundation/aztec-address';

export type ProtocolContractAddresses = {
  classRegisterer: AztecAddress;
  gasToken: AztecAddress;
  instanceDeployer: AztecAddress;
  keyRegistry: AztecAddress;
  multiCallEntrypoint: AztecAddress;
};
