import { DefaultWaitOpts, type EthAddress, NoFeePaymentMethod, type Wallet } from '@aztec/aztec.js';
import {
  AztecAddress,
  CANONICAL_AUTH_REGISTRY_ADDRESS,
  CANONICAL_KEY_REGISTRY_ADDRESS,
  GasSettings,
  MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS,
} from '@aztec/circuits.js';
import { bufferAsFields } from '@aztec/foundation/abi';
import { getCanonicalAuthRegistry } from '@aztec/protocol-contracts/auth-registry';
import { getCanonicalFeeJuice } from '@aztec/protocol-contracts/fee-juice';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';

/**
 * Deploys the contract to pay for gas on L2.
 */
export async function deployCanonicalL2FeeJuice(
  deployer: Wallet,
  feeJuicePortalAddress: EthAddress,
  waitOpts = DefaultWaitOpts,
): Promise<AztecAddress> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { FeeJuiceContract } = await import('@aztec/noir-contracts.js');

  const canonicalFeeJuice = getCanonicalFeeJuice();

  if (await deployer.isContractClassPubliclyRegistered(canonicalFeeJuice.contractClass.id)) {
    return canonicalFeeJuice.address;
  }

  const publicBytecode = canonicalFeeJuice.contractClass.packedBytecode;
  const encodedBytecode = bufferAsFields(publicBytecode, MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS);
  await deployer.addCapsule(encodedBytecode);
  const feeJuiceContract = await FeeJuiceContract.at(canonicalFeeJuice.address, deployer);
  await feeJuiceContract.methods
    .deploy(
      canonicalFeeJuice.contractClass.artifactHash,
      canonicalFeeJuice.contractClass.privateFunctionsRoot,
      canonicalFeeJuice.contractClass.publicBytecodeCommitment,
      feeJuicePortalAddress,
    )
    .send({ fee: { paymentMethod: new NoFeePaymentMethod(), gasSettings: GasSettings.teardownless() } })
    .wait(waitOpts);

  if (!feeJuiceContract.address.equals(canonicalFeeJuice.address)) {
    throw new Error(
      `Deployed Fee Juice address ${feeJuiceContract.address} does not match expected address ${canonicalFeeJuice.address}`,
    );
  }

  if (!(await deployer.isContractPubliclyDeployed(canonicalFeeJuice.address))) {
    throw new Error(`Failed to deploy Fee Juice to ${canonicalFeeJuice.address}`);
  }

  return canonicalFeeJuice.address;
}

/**
 * Deploys the key registry on L2.
 */
export async function deployCanonicalKeyRegistry(deployer: Wallet, waitOpts = DefaultWaitOpts): Promise<AztecAddress> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing noir-contracts.js even in devDeps results in a circular dependency error. Need to ignore because this line doesn't cause an error in a dev environment
  const { NewKeyRegistryContract: KeyRegistryContract } = await import('@aztec/noir-contracts.js');

  const canonicalKeyRegistry = getCanonicalKeyRegistry();

  // We check to see if there exists a contract at the canonical Key Registry address with the same contract class id as we expect. This means that
  // the key registry has already been deployed to the correct address.
  if (
    (await deployer.getContractInstance(canonicalKeyRegistry.address))?.contractClassId.equals(
      canonicalKeyRegistry.contractClass.id,
    ) &&
    (await deployer.isContractClassPubliclyRegistered(canonicalKeyRegistry.contractClass.id))
  ) {
    return canonicalKeyRegistry.address;
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

  return canonicalKeyRegistry.address;
}

/**
 * Deploys the auth registry on L2.
 */
export async function deployCanonicalAuthRegistry(deployer: Wallet, waitOpts = DefaultWaitOpts): Promise<AztecAddress> {
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
    return canonicalAuthRegistry.address;
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

  return canonicalAuthRegistry.address;
}
