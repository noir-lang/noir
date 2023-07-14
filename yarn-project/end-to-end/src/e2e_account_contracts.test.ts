import { AztecRPCServer } from '@aztec/aztec-rpc';
import {
  AccountImplementation,
  AccountWallet,
  Contract,
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
import { ChildAbi, EcdsaAccountContractAbi, SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';
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
  privateKey: Buffer,
  createWallet: CreateAccountImplFn,
) {
  const salt = Fr.random();
  const publicKey = await generatePublicKey(privateKey);
  const { address, partialAddress } = await getContractDeploymentInfo(abi, args, salt, publicKey);
  await aztecRpcServer.addAccount(privateKey, address, partialAddress);
  await deployContract(aztecRpcServer, publicKey, abi, args, salt);
  const wallet = new AccountWallet(aztecRpcServer, await createWallet(address, partialAddress, privateKey));
  return { wallet, address, partialAddress };
}

type CreateAccountImplFn = (
  address: AztecAddress,
  partialAddress: PartialContractAddress,
  privateKey: Buffer,
) => Promise<AccountImplementation>;

function itShouldBehaveLikeAnAccountContract(abi: ContractAbi, argsFn: () => any[], createWallet: CreateAccountImplFn) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: Contract;
    let address: AztecAddress;
    let partialAddress: PartialContractAddress;
    let wallet: AccountWallet;

    beforeEach(async () => {
      context = await setup();
      const privateKey = randomBytes(32);
      const { aztecRpcServer } = context;
      ({ wallet, address, partialAddress } = await createNewAccount(
        aztecRpcServer,
        abi,
        argsFn(),
        privateKey,
        createWallet,
      ));

      const { address: childAddress } = await deployContract(aztecRpcServer, Point.random(), ChildAbi, []);
      child = new Contract(childAddress, ChildAbi, wallet);
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
        await createWallet(address, partialAddress, randomBytes(32)),
      );
      const childWithInvalidWallet = new Contract(child.address, child.abi, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).simulate()).rejects.toThrowError(
        /could not satisfy all constraints/,
      );
    });
  });
}

describe('e2e_account_contracts', () => {
  describe('schnorr account', () => {
    const createSchnorrWallet = async (address: AztecAddress, partial: PartialContractAddress, privateKey: Buffer) =>
      new SingleKeyAccountContract(address, partial, privateKey, await Schnorr.new());

    itShouldBehaveLikeAnAccountContract(SchnorrAccountContractAbi, () => [], createSchnorrWallet);
  });

  describe.skip('ecdsa account', () => {
    const createEcdsaWallet = async (address: AztecAddress, _partial: PartialContractAddress, privateKey: Buffer) =>
      new StoredKeyAccountContract(address, privateKey, await Ecdsa.new());

    let ecdsaPrivateKey: Buffer;
    let ecdsaPublicKey: Buffer;
    let ecdsaCreateArgs: any[];

    beforeAll(async () => {
      ecdsaPrivateKey = randomBytes(32);
      const ecdsa = await Ecdsa.new();
      ecdsaPublicKey = ecdsa.computePublicKey(ecdsaPrivateKey);
      ecdsaCreateArgs = [ecdsaPublicKey.subarray(0, 32), ecdsaPublicKey.subarray(32, 64)];
    });

    itShouldBehaveLikeAnAccountContract(EcdsaAccountContractAbi, () => ecdsaCreateArgs, createEcdsaWallet);
  });
});
