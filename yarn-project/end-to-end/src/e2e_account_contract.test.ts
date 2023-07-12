import { AztecNodeService } from '@aztec/aztec-node';
import {
  AccountContract,
  AccountWallet,
  Contract,
  ContractDeployer,
  EcdsaAuthProvider,
  Fr,
  SchnorrAuthProvider,
  Wallet,
  generatePublicKey,
} from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { ChildAbi, EcdsaAccountContractAbi, SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';
import { Ecdsa, Schnorr } from '@aztec/circuits.js/barretenberg';
import { CircuitsWasm, getContractDeploymentInfo, Point } from '@aztec/circuits.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { randomBytes } from 'crypto';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { PublicKey, TxStatus } from '@aztec/types';

import { privateKey2 } from './fixtures.js';
import { setup } from './utils.js';

describe('e2e_account_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let logger: DebugLogger;
  let child: Contract;

  const sendContractDeployment = async (publicKey: PublicKey, abi: ContractAbi, contractAddressSalt: Fr) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer, publicKey);
    const deployMethod = deployer.deploy();
    await deployMethod.create({ contractAddressSalt });
    const tx = deployMethod.send();
    expect(await tx.isMined(0, 0.1)).toBeTruthy();

    return { tx, partialContractAddress: deployMethod.partialContractAddress! };
  };

  const deployAccountContract = async (
    abi: ContractAbi,
    authProvider: EcdsaAuthProvider | SchnorrAuthProvider,
    publicKey: PublicKey,
    privateKey: Buffer,
  ) => {
    const contractAddressSalt = Fr.random();
    const contractDeploymentInfo = await getContractDeploymentInfo(abi, [], contractAddressSalt, publicKey);
    await aztecRpcServer.addAccount(
      privateKey,
      contractDeploymentInfo.address,
      contractDeploymentInfo.partialAddress,
      abi,
    );
    const accountDeploymentTx = await sendContractDeployment(publicKey, abi, contractAddressSalt);
    expect(await accountDeploymentTx.tx.isMined(0, 0.1)).toBeTruthy();

    const wallet = new AccountWallet(
      aztecRpcServer,
      new AccountContract(
        contractDeploymentInfo.address,
        publicKey,
        authProvider,
        contractDeploymentInfo.partialAddress,
        abi,
        await CircuitsWasm.get(),
      ),
    );

    return {
      contractAddress: contractDeploymentInfo.address,
      wallet,
    };
  };

  const deployChildContract = async (publicKey: Point, wallet: Wallet) => {
    const contractAddressSalt = Fr.random();
    const childDeployTx = await sendContractDeployment(publicKey, ChildAbi, contractAddressSalt);
    await childDeployTx.tx.isMined(0, 0.1);
    const childReceipt = await childDeployTx.tx.getReceipt();
    expect(childReceipt.status).toEqual(TxStatus.MINED);
    return new Contract(childReceipt.contractAddress!, ChildAbi, wallet);
  };

  const deployAll = async () => {
    logger('Deploying L2 contracts using schnorr account contract...');
    const schnorrPublicKey = await generatePublicKey(privateKey2);
    const { contractAddress: schnorrAccountContractAddress, wallet: schnorrWallet } = await deployAccountContract(
      SchnorrAccountContractAbi,
      new SchnorrAuthProvider(await Schnorr.new(), privateKey2),
      schnorrPublicKey,
      privateKey2,
    );
    logger('Deploying L2 contracts using ecdsa account contract...');
    const ecdsaPublicKey = Point.fromBuffer((await Ecdsa.new()).computePublicKey(privateKey2));
    const { contractAddress: ecdsaAccountContractAddress, wallet: ecdsaWallet } = await deployAccountContract(
      EcdsaAccountContractAbi,
      new EcdsaAuthProvider(privateKey2),
      ecdsaPublicKey,
      privateKey2,
    );
    logger('Deploying child contract...');
    child = await deployChildContract(
      await schnorrWallet.getAccountPublicKey(schnorrAccountContractAddress),
      schnorrWallet,
    );
    logger(
      `Schnorr contract at ${schnorrAccountContractAddress.toString()}, ecdsa contract at ${ecdsaAccountContractAddress.toString()}, child contract at ${child.address.toString()}`,
    );
    // create a contract object to be used with the ecdsa signer
    const childContractWithEcdsaSigning = new Contract(child.address, child.abi, ecdsaWallet);

    return {
      child,
      childContractWithEcdsaSigning,
      schnorrAccountContractAddress,
      ecdsaAccountContractAddress,
    };
  };

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup(0));
  }, 100_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  it('calls a private function', async () => {
    const { child, childContractWithEcdsaSigning, schnorrAccountContractAddress, ecdsaAccountContractAddress } =
      await deployAll();

    logger('Calling private function...');
    const tx1 = child.methods.value(42).send({ origin: schnorrAccountContractAddress });
    const tx2 = childContractWithEcdsaSigning.methods.value(56).send({ origin: ecdsaAccountContractAddress });

    const txs = [tx1, tx2];

    await Promise.all(txs.map(tx => tx.isMined(0, 0.1)));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));

    expect(receipts[0].status).toBe(TxStatus.MINED);
    expect(receipts[1].status).toBe(TxStatus.MINED);
  }, 60_000);

  it('calls a public function', async () => {
    const { child, childContractWithEcdsaSigning, schnorrAccountContractAddress, ecdsaAccountContractAddress } =
      await deployAll();

    logger('Calling public function...');
    const tx1 = child.methods.pubStoreValue(42).send({ origin: schnorrAccountContractAddress });
    const tx2 = childContractWithEcdsaSigning.methods.pubStoreValue(15).send({ origin: ecdsaAccountContractAddress });

    const txs = [tx1, tx2];

    await Promise.all(txs.map(tx => tx.isMined(0, 0.1)));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));

    expect(receipts[0].status).toBe(TxStatus.MINED);
    expect(receipts[1].status).toBe(TxStatus.MINED);
    // The contract accumulates the values so the expected value is 95
    expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(57n);
  }, 60_000);

  it('fails to execute function with invalid schnorr signature', async () => {
    logger('Deploying L2 contracts with invalid Schnorr signer...');
    const schnorrPublicKey = await generatePublicKey(privateKey2);
    const { contractAddress: schnorrAccountContractAddress, wallet: schnorrWallet } = await deployAccountContract(
      SchnorrAccountContractAbi,
      new SchnorrAuthProvider(await Schnorr.new(), randomBytes(32)),
      schnorrPublicKey,
      privateKey2,
    );
    logger('Deploying child contract...');
    child = await deployChildContract(
      await schnorrWallet.getAccountPublicKey(schnorrAccountContractAddress),
      schnorrWallet,
    );
    await expect(child.methods.value(42).simulate({ origin: schnorrAccountContractAddress })).rejects.toThrowError(
      /could not satisfy all constraints/,
    );
  }, 60_000);

  it('fails to execute function with invalid ecdsa signature', async () => {
    logger('Deploying L2 contracts with invalid ecdsa signer...');
    const ecdsaPublicKey = Point.fromBuffer((await Ecdsa.new()).computePublicKey(privateKey2));
    const { contractAddress: ecdsaAccountContractAddress, wallet: ecdsaWallet } = await deployAccountContract(
      EcdsaAccountContractAbi,
      new EcdsaAuthProvider(randomBytes(32)),
      ecdsaPublicKey,
      privateKey2,
    );
    logger('Deploying child contract...');
    child = await deployChildContract(await ecdsaWallet.getAccountPublicKey(ecdsaAccountContractAddress), ecdsaWallet);
    await expect(child.methods.value(42).simulate({ origin: ecdsaAccountContractAddress })).rejects.toThrowError(
      /could not satisfy all constraints/,
    );
  }, 60_000);
});
