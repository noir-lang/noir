import { AztecAddress, SignerlessWallet, createPXEClient } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { CANONICAL_KEY_REGISTRY_ADDRESS } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';
import { GasTokenContract, KeyRegistryContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';

export async function bootstrap(rpcUrl: string, log: LogFn) {
  const pxe = createPXEClient(rpcUrl);
  const canonicalKeyRegistry = getCanonicalKeyRegistry();
  const deployer = new SignerlessWallet(pxe, new DefaultMultiCallEntrypoint(31337, 1));

  if (
    (await deployer.getContractInstance(canonicalKeyRegistry.address))?.contractClassId.equals(
      canonicalKeyRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalKeyRegistry.contractClass.id))
  ) {
    log('Key Registry already deployed');
    return;
  }

  const keyRegistry = await KeyRegistryContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalKeyRegistry.instance.salt, universalDeploy: true })
    .deployed();

  if (
    !keyRegistry.address.equals(canonicalKeyRegistry.address) ||
    !keyRegistry.address.equals(AztecAddress.fromBigInt(CANONICAL_KEY_REGISTRY_ADDRESS))
  ) {
    throw new Error(
      `Deployed Key Registry address ${keyRegistry.address} does not match expected address ${canonicalKeyRegistry.address}, or they both do not equal CANONICAL_KEY_REGISTRY_ADDRESS`,
    );
  }

  log(`Key Registry deployed at canonical address ${keyRegistry.address.toString()}`);

  const gasPortalAddress = (await deployer.getNodeInfo()).l1ContractAddresses.gasPortalAddress;
  const canonicalGasToken = getCanonicalGasToken();

  if (await deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    log('Gas token already deployed');
    return;
  }

  const gasToken = await GasTokenContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalGasToken.instance.salt, universalDeploy: true })
    .deployed();
  await gasToken.methods.set_portal(gasPortalAddress).send().wait();

  log(`Gas token deployed at canonical address ${gasToken.address.toString()}`);
}
