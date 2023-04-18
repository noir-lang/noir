import { AztecAddress, Fr } from '@aztec/foundation';

export interface PublicDB {
  storageRead(contract: AztecAddress, slot: Fr): Promise<Fr>;
  storageWrite(contract: AztecAddress, slot: Fr, value: Fr): Promise<Fr>;
}
