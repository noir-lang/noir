import { SignerlessWallet, type WaitOpts, createPXEClient, makeFetch } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { type LogFn } from '@aztec/foundation/log';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';

const waitOpts: WaitOpts = {
  timeout: 1800,
  interval: 1,
};

export async function bootstrap(rpcUrl: string, log: LogFn) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { GasTokenContract, KeyRegistryContract } = await import('@aztec/noir-contracts.js');

  const pxe = createPXEClient(rpcUrl, makeFetch([], true));
  const deployer = new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(31337, 1));

  const canonicalKeyRegistry = getCanonicalKeyRegistry();
  const keyRegistryDeployParams = {
    contractAddressSalt: canonicalKeyRegistry.instance.salt,
    universalDeploy: true,
  };
  const keyRegistryTx = KeyRegistryContract.deploy(deployer);

  const gasPortalAddress = (await deployer.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken();
  const gasTokenDeployParams = {
    contractAddressSalt: canonicalGasToken.instance.salt,
    universalDeploy: true,
  };
  const gasTokenTx = GasTokenContract.deploy(deployer);

  // prove these txs sequentially otherwise global fetch with default options times out with real proofs
  await keyRegistryTx.prove(keyRegistryDeployParams);
  const keyRegistry = await keyRegistryTx.send(keyRegistryDeployParams).deployed(waitOpts);

  await gasTokenTx.prove(gasTokenDeployParams);
  // also deploy the accounts sequentially otherwise there's too much data and publishing TxEffects fails
  const gasToken = await gasTokenTx.send(gasTokenDeployParams).deployed(waitOpts);

  log(`Key Registry deployed at canonical address ${keyRegistry.address.toString()}`);
  log(`Gas token deployed at canonical address ${gasToken.address.toString()}`);

  const portalSetTx = gasToken.methods.set_portal(gasPortalAddress);
  await portalSetTx.prove();
  portalSetTx.send();
}
