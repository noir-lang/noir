import { L1Addresses } from '@aztec/l1-contracts';

export interface Config extends L1Addresses {
  sequencerPrivateKey: string;
  ethereumHost: string;
  requiredConfirmations: number;
}
