import { SignerlessWallet, type WaitOpts, createPXEClient, makeFetch } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { type LogFn } from '@aztec/foundation/log';

import {
  deployCanonicalAuthRegistry,
  deployCanonicalKeyRegistry,
  deployCanonicalL2GasToken,
} from '../utils/deploy_contracts.js';

const waitOpts: WaitOpts = {
  timeout: 180,
  interval: 1,
};

export async function bootstrap(rpcUrl: string, l1ChainId: number, log: LogFn) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  // const { TokenContract } = await import('@aztec/noir-contracts.js');
  const pxe = createPXEClient(rpcUrl, makeFetch([], true));
  const deployer = new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(l1ChainId, 1));

  // Deploy Key Registry
  await deployCanonicalKeyRegistry(deployer, log, waitOpts);

  // Deploy Auth Registry
  await deployCanonicalAuthRegistry(deployer, log, waitOpts);

  // Deploy Fee Juice
  const gasPortalAddress = (await deployer.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  await deployCanonicalL2GasToken(deployer, gasPortalAddress, log, waitOpts);
}
