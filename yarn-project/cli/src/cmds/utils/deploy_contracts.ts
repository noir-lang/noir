import { DefaultWaitOpts, type EthAddress, NoFeePaymentMethod, type Wallet } from '@aztec/aztec.js';
import {
  AztecAddress,
  CANONICAL_AUTH_REGISTRY_ADDRESS,
  CANONICAL_KEY_REGISTRY_ADDRESS,
  GasSettings,
  MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS,
} from '@aztec/circuits.js';
import { bufferAsFields } from '@aztec/foundation/abi';
import { type LogFn } from '@aztec/foundation/log';
import { getCanonicalAuthRegistry } from '@aztec/protocol-contracts/auth-registry';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';

/**
 * Deploys the contract to pay for gas on L2.
 */
export async function deployCanonicalL2GasToken(
  deployer: Wallet,
  gasPortalAddress: EthAddress,
  log: LogFn,
  waitOpts = DefaultWaitOpts,
) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { GasTokenContract } = await import('@aztec/noir-contracts.js');

  const canonicalGasToken = getCanonicalGasToken();

  if (await deployer.isContractClassPubliclyRegistered(canonicalGasToken.contractClass.id)) {
    log(`Gas Token contract class already registered with id ${canonicalGasToken.contractClass.id}`);
    return;
  }

  const publicBytecode = canonicalGasToken.contractClass.packedBytecode;
  const encodedBytecode = bufferAsFields(publicBytecode, MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS);
  await deployer.addCapsule(encodedBytecode);
  const gasToken = await GasTokenContract.at(canonicalGasToken.address, deployer);
  await gasToken.methods
    .deploy(
      canonicalGasToken.contractClass.artifactHash,
      canonicalGasToken.contractClass.privateFunctionsRoot,
      canonicalGasToken.contractClass.publicBytecodeCommitment,
      gasPortalAddress,
    )
    .send({ fee: { paymentMethod: new NoFeePaymentMethod(), gasSettings: GasSettings.teardownless() } })
    .wait(waitOpts);

  if (!gasToken.address.equals(canonicalGasToken.address)) {
    throw new Error(
      `Deployed Gas Token address ${gasToken.address} does not match expected address ${canonicalGasToken.address}`,
    );
  }

  if (!(await deployer.isContractPubliclyDeployed(canonicalGasToken.address))) {
    throw new Error(`Failed to deploy Gas Token to ${canonicalGasToken.address}`);
  }

  log(`Deployed Gas Token on L2 at ${canonicalGasToken.address}`);
}

/**
 * Deploys the key registry on L2.
 */
export async function deployCanonicalKeyRegistry(deployer: Wallet, log: LogFn, waitOpts = DefaultWaitOpts) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { KeyRegistryContract } = await import('@aztec/noir-contracts.js');

  const canonicalKeyRegistry = getCanonicalKeyRegistry();

  // We check to see if there exists a contract at the canonical Key Registry address with the same contract class id as we expect. This means that
  // the key registry has already been deployed to the correct address.
  if (
    (await deployer.getContractInstance(canonicalKeyRegistry.address))?.contractClassId.equals(
      canonicalKeyRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalKeyRegistry.contractClass.id))
  ) {
    log(`Key Registry already deployed at ${canonicalKeyRegistry.address}`);
    return;
  }

  const keyRegistry = await KeyRegistryContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalKeyRegistry.instance.salt, universalDeploy: true })
    .deployed(waitOpts);

  if (
    !keyRegistry.address.equals(canonicalKeyRegistry.address) ||
    !keyRegistry.address.equals(AztecAddress.fromBigInt(CANONICAL_KEY_REGISTRY_ADDRESS))
  ) {
    throw new Error(
      `Deployed Key Registry address ${keyRegistry.address} does not match expected address ${canonicalKeyRegistry.address}, or they both do not equal CANONICAL_KEY_REGISTRY_ADDRESS`,
    );
  }

  log(`Deployed Key Registry on L2 at ${canonicalKeyRegistry.address}`);
}

/**
 * Deploys the auth registry on L2.
 */
export async function deployCanonicalAuthRegistry(deployer: Wallet, log: LogFn, waitOpts = DefaultWaitOpts) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { AuthRegistryContract } = await import('@aztec/noir-contracts.js');

  const canonicalAuthRegistry = getCanonicalAuthRegistry();

  // We check to see if there exists a contract at the canonical Auth Registry address with the same contract class id as we expect. This means that
  // the auth registry has already been deployed to the correct address.
  if (
    (await deployer.getContractInstance(canonicalAuthRegistry.address))?.contractClassId.equals(
      canonicalAuthRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalAuthRegistry.contractClass.id))
  ) {
    log(`Auth Registry already deployed at ${canonicalAuthRegistry.address}`);
    return;
  }

  const authRegistry = await AuthRegistryContract.deploy(deployer)
    .send({ contractAddressSalt: canonicalAuthRegistry.instance.salt, universalDeploy: true })
    .deployed(waitOpts);

  if (
    !authRegistry.address.equals(canonicalAuthRegistry.address) ||
    !authRegistry.address.equals(AztecAddress.fromBigInt(CANONICAL_AUTH_REGISTRY_ADDRESS))
  ) {
    throw new Error(
      `Deployed Auth Registry address ${authRegistry.address} does not match expected address ${canonicalAuthRegistry.address}, or they both do not equal CANONICAL_AUTH_REGISTRY_ADDRESS`,
    );
  }

  log(`Deployed Auth Registry on L2 at ${canonicalAuthRegistry.address}`);
}
