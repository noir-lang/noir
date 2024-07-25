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

export async function deployProtocolContracts(rpcUrl: string, l1ChainId: number, json: boolean, log: LogFn) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  // const { TokenContract } = await import('@aztec/noir-contracts.js');
  const pxe = createPXEClient(rpcUrl, makeFetch([], true));
  const deployer = new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(l1ChainId, 1));

  // Deploy Key Registry
  const keyRegistryAddress = await deployCanonicalKeyRegistry(deployer, waitOpts);

  // Deploy Auth Registry
  const authRegistryAddress = await deployCanonicalAuthRegistry(deployer, waitOpts);

  // Deploy Fee Juice
  const gasPortalAddress = (await deployer.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  const feeJuiceAddress = await deployCanonicalL2GasToken(deployer, gasPortalAddress, waitOpts);

  if (json) {
    log(
      JSON.stringify(
        {
          keyRegistryAddress: keyRegistryAddress.toString(),
          authRegistryAddress: authRegistryAddress.toString(),
          feeJuiceAddress: feeJuiceAddress.toString(),
        },
        null,
        2,
      ),
    );
  } else {
    log(`Key Registry: ${keyRegistryAddress}`);
    log(`Auth Registry: ${authRegistryAddress}`);
    log(`Fee Juice: ${feeJuiceAddress}`);
  }
}
