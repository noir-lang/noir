import { AztecRPCServer } from '@aztec/aztec-rpc';
import {
  AccountImplementation,
  AccountWallet,
  ContractDeployer,
  Fr,
  SingleKeyAccountContract,
  StoredKeyAccountContract,
  generatePublicKey,
} from '@aztec/aztec.js';
import { AztecAddress, PartialContractAddress, Point, getContractDeploymentInfo } from '@aztec/circuits.js';
import { Ecdsa, Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { toBigInt } from '@aztec/foundation/serialize';
import {
  EcdsaAccountContractAbi,
  SchnorrMultiKeyAccountContractAbi,
  SchnorrSingleKeyAccountContractAbi,
} from '@aztec/noir-contracts/examples';
import { ChildContract } from '@aztec/noir-contracts/types';
import { PublicKey } from '@aztec/types';

import { randomBytes } from 'crypto';

import { setup } from './utils.js';

async function deployContract(
  aztecRpcServer: AztecRPCServer,
  publicKey: PublicKey,
  abi: ContractAbi,
  args: any[],
  contractAddressSalt?: Fr,
) {
  const deployer = new ContractDeployer(abi, aztecRpcServer, publicKey);
  const deployMethod = deployer.deploy(...args);
  await deployMethod.create({ contractAddressSalt });
  const tx = deployMethod.send();
  expect(await tx.isMined(0, 0.1)).toBeTruthy();
  const receipt = await tx.getReceipt();
  return { address: receipt.contractAddress!, partialContractAddress: deployMethod.partialContractAddress! };
}

async function createNewAccount(
  aztecRpcServer: AztecRPCServer,
  abi: ContractAbi,
  args: any[],
  encryptionPrivateKey: Buffer,
  useProperKey: boolean,
  createAccountImpl: CreateAccountImplFn,
) {
  const salt = Fr.random();
  const publicKey = await generatePublicKey(encryptionPrivateKey);
  const { address, partialAddress } = await getContractDeploymentInfo(abi, args, salt, publicKey);
  await aztecRpcServer.addAccount(encryptionPrivateKey, address, partialAddress);
  await deployContract(aztecRpcServer, publicKey, abi, args, salt);
  const account = await createAccountImpl(address, useProperKey, partialAddress, encryptionPrivateKey);
  const wallet = new AccountWallet(aztecRpcServer, account);
  return { wallet, address, partialAddress };
}

type CreateAccountImplFn = (
  address: AztecAddress,
  useProperKey: boolean,
  partialAddress: PartialContractAddress,
  encryptionPrivateKey: Buffer,
) => Promise<AccountImplementation>;

function itShouldBehaveLikeAnAccountContract(
  abi: ContractAbi,
  argsFn: () => any[],
  createAccountImpl: CreateAccountImplFn,
) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: ChildContract;
    let address: AztecAddress;
    let partialAddress: PartialContractAddress;
    let wallet: AccountWallet;
    let encryptionPrivateKey: Buffer;

    beforeEach(async () => {
      context = await setup();
      encryptionPrivateKey = randomBytes(32);
      const { aztecRpcServer } = context;
      ({ wallet, address, partialAddress } = await createNewAccount(
        aztecRpcServer,
        abi,
        argsFn(),
        encryptionPrivateKey,
        true,
        createAccountImpl,
      ));

      const { address: childAddress } = await deployContract(aztecRpcServer, Point.random(), ChildContract.abi, []);
      child = new ChildContract(childAddress, wallet);
    }, 60_000);

    afterEach(async () => {
      await context.aztecNode.stop();
      await context.aztecRpcServer.stop();
    });

    it('calls a private function', async () => {
      const { logger } = context;
      logger('Calling private function...');
      const tx = child.methods.value(42).send();
      expect(await tx.isMined(0, 0.1)).toBeTruthy();
    }, 60_000);

    it('calls a public function', async () => {
      const { logger, aztecNode } = context;
      logger('Calling public function...');
      const tx = child.methods.pubStoreValue(42).send();
      expect(await tx.isMined(0, 0.1)).toBeTruthy();
      expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(42n);
    }, 60_000);

    it('fails to call a function using an invalid signature', async () => {
      const invalidWallet = new AccountWallet(
        context.aztecRpcServer,
        await createAccountImpl(address, false, partialAddress, encryptionPrivateKey),
      );
      const childWithInvalidWallet = new ChildContract(child.address, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).simulate()).rejects.toThrowError(
        /could not satisfy all constraints/,
      );
    });
  });
}

describe('e2e_account_contracts', () => {
  describe('schnorr single-key account', () => {
    const createWallet = async (
      address: AztecAddress,
      useProperKey: boolean,
      partial: PartialContractAddress,
      privateKey: Buffer,
    ) =>
      new SingleKeyAccountContract(address, partial, useProperKey ? privateKey : randomBytes(32), await Schnorr.new());

    itShouldBehaveLikeAnAccountContract(SchnorrSingleKeyAccountContractAbi, () => [], createWallet);
  });

  describe('schnorr multi-key account', () => {
    let signingPrivateKey: Buffer;
    let signingPublicKey: Buffer;
    let createArgs: any[];

    const createWallet = async (address: AztecAddress, useProperKey: boolean) =>
      new StoredKeyAccountContract(address, useProperKey ? signingPrivateKey : randomBytes(32), await Schnorr.new());

    beforeAll(async () => {
      signingPrivateKey = randomBytes(32);
      const schnorr = await Schnorr.new();
      signingPublicKey = schnorr.computePublicKey(signingPrivateKey);
      createArgs = [Fr.fromBuffer(signingPublicKey.subarray(0, 32)), Fr.fromBuffer(signingPublicKey.subarray(32, 64))];
    });

    itShouldBehaveLikeAnAccountContract(SchnorrMultiKeyAccountContractAbi, () => createArgs, createWallet);
  });

  describe('ecdsa stored-key account', () => {
    let ecdsaPrivateKey: Buffer;
    let ecdsaPublicKey: Buffer;
    let ecdsaCreateArgs: any[];

    const createWallet = async (address: AztecAddress, useProperKey: boolean) =>
      new StoredKeyAccountContract(address, useProperKey ? ecdsaPrivateKey : randomBytes(32), await Ecdsa.new());

    beforeAll(async () => {
      ecdsaPrivateKey = randomBytes(32);
      const ecdsa = await Ecdsa.new();
      ecdsaPublicKey = ecdsa.computePublicKey(ecdsaPrivateKey);
      ecdsaCreateArgs = [ecdsaPublicKey.subarray(0, 32), ecdsaPublicKey.subarray(32, 64)];
    });

    itShouldBehaveLikeAnAccountContract(EcdsaAccountContractAbi, () => ecdsaCreateArgs, createWallet);
  });
});
