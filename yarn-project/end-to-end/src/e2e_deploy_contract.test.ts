import {
  AztecAddress,
  AztecNode,
  BatchCall,
  CompleteAddress,
  Contract,
  ContractArtifact,
  ContractBase,
  ContractClassWithId,
  ContractDeployer,
  ContractInstanceWithAddress,
  DebugLogger,
  EthAddress,
  Fr,
  PXE,
  SignerlessWallet,
  TxHash,
  TxStatus,
  Wallet,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
  isContractDeployed,
} from '@aztec/aztec.js';
import {
  broadcastPrivateFunction,
  broadcastUnconstrainedFunction,
  deployInstance,
  registerContractClass,
} from '@aztec/aztec.js/deployment';
import { ContractClassIdPreimage, Point, PublicKey } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { ContractInstanceDeployerContract, StatefulTestContract } from '@aztec/noir-contracts.js';
import { TestContract, TestContractArtifact } from '@aztec/noir-contracts.js/Test';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
import { SequencerClient } from '@aztec/sequencer-client';

import { setup } from './fixtures/utils.js';

describe('e2e_deploy_contract', () => {
  let pxe: PXE;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let wallet: Wallet;
  let sequencer: SequencerClient | undefined;
  let aztecNode: AztecNode;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, pxe, accounts, logger, wallet, sequencer, aztecNode } = await setup());
  }, 100_000);

  afterAll(() => teardown());

  /**
   * Milestone 1.1.
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it('should deploy a contract', async () => {
    const publicKey = accounts[0].publicKey;
    const salt = Fr.random();
    const deploymentData = getContractInstanceFromDeployParams(
      TestContractArtifact,
      [],
      salt,
      publicKey,
      EthAddress.ZERO,
    );
    const deployer = new ContractDeployer(TestContractArtifact, pxe, publicKey);
    const tx = deployer.deploy().send({ contractAddressSalt: salt });
    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        status: TxStatus.PENDING,
        error: '',
      }),
    );
    logger(`Receipt received and expecting contract deployment at ${receipt.contractAddress}`);
    // we pass in wallet to wait(...) because wallet is necessary to create a TS contract instance
    const receiptAfterMined = await tx.wait({ wallet });

    expect(receiptAfterMined).toEqual(
      expect.objectContaining({
        status: TxStatus.MINED,
        error: '',
        contractAddress: deploymentData.address,
      }),
    );
    const contractAddress = receiptAfterMined.contractAddress!;
    expect(await isContractDeployed(pxe, contractAddress)).toBe(true);
    expect(await isContractDeployed(pxe, AztecAddress.random())).toBe(false);
  }, 60_000);

  /**
   * Verify that we can produce multiple rollups.
   */
  it('should deploy one contract after another in consecutive rollups', async () => {
    const deployer = new ContractDeployer(TestContractArtifact, pxe);

    for (let index = 0; index < 2; index++) {
      logger(`Deploying contract ${index + 1}...`);
      // we pass in wallet to wait(...) because wallet is necessary to create a TS contract instance
      const receipt = await deployer.deploy().send({ contractAddressSalt: Fr.random() }).wait({ wallet });
      expect(receipt.status).toBe(TxStatus.MINED);
    }
  }, 60_000);

  /**
   * Verify that we can deploy multiple contracts and interact with all of them.
   */
  it('should deploy multiple contracts and interact with them', async () => {
    const deployer = new ContractDeployer(TestContractArtifact, pxe);

    for (let index = 0; index < 2; index++) {
      logger(`Deploying contract ${index + 1}...`);
      const receipt = await deployer.deploy().send({ contractAddressSalt: Fr.random() }).wait({ wallet });

      const contract = await Contract.at(receipt.contractAddress!, TestContractArtifact, wallet);
      logger(`Sending TX to contract ${index + 1}...`);
      await contract.methods.get_public_key(accounts[0].address).send().wait();
    }
  }, 60_000);

  /**
   * Milestone 1.2.
   * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
   */
  it('should not deploy a contract with the same salt twice', async () => {
    const contractAddressSalt = Fr.random();
    const deployer = new ContractDeployer(TestContractArtifact, pxe);

    {
      // we pass in wallet to wait(...) because wallet is necessary to create a TS contract instance
      const receipt = await deployer.deploy().send({ contractAddressSalt }).wait({ wallet });

      expect(receipt.status).toBe(TxStatus.MINED);
      expect(receipt.error).toBe('');
    }

    {
      await expect(deployer.deploy().send({ contractAddressSalt }).wait()).rejects.toThrowError(
        /A settled tx with equal hash/,
      );
    }
  }, 60_000);

  it('should deploy a contract connected to a portal contract', async () => {
    const deployer = new ContractDeployer(TestContractArtifact, wallet);
    const portalContract = EthAddress.random();

    // ContractDeployer was instantiated with wallet so we don't have to pass it to wait(...)
    const txReceipt = await deployer.deploy().send({ portalContract }).wait();

    expect(txReceipt.status).toBe(TxStatus.MINED);
    const contractAddress = txReceipt.contractAddress!;

    expect((await pxe.getContractData(contractAddress))?.portalContractAddress.toString()).toEqual(
      portalContract.toString(),
    );
    expect((await pxe.getExtendedContractData(contractAddress))?.contractData.portalContractAddress.toString()).toEqual(
      portalContract.toString(),
    );
  }, 60_000);

  it('it should not deploy a contract which failed the public part of the execution', async () => {
    sequencer?.updateSequencerConfig({
      minTxsPerBlock: 2,
    });

    try {
      // This test requires at least another good transaction to go through in the same block as the bad one.
      // I deployed the same contract again but it could really be any valid transaction here.
      const goodDeploy = new ContractDeployer(TokenContractArtifact, wallet).deploy(
        AztecAddress.random(),
        'TokenName',
        'TKN',
        18,
      );
      const badDeploy = new ContractDeployer(TokenContractArtifact, wallet).deploy(
        AztecAddress.ZERO,
        'TokenName',
        'TKN',
        18,
      );

      await Promise.all([
        goodDeploy.simulate({ skipPublicSimulation: true }),
        badDeploy.simulate({ skipPublicSimulation: true }),
      ]);

      const [goodTx, badTx] = [
        goodDeploy.send({ skipPublicSimulation: true }),
        badDeploy.send({ skipPublicSimulation: true }),
      ];

      const [goodTxPromiseResult, badTxReceiptResult] = await Promise.allSettled([goodTx.wait(), badTx.wait()]);

      expect(goodTxPromiseResult.status).toBe('fulfilled');
      expect(badTxReceiptResult.status).toBe('rejected');

      const [goodTxReceipt, badTxReceipt] = await Promise.all([goodTx.getReceipt(), badTx.getReceipt()]);

      expect(goodTxReceipt.blockNumber).toEqual(expect.any(Number));
      expect(badTxReceipt.blockNumber).toBeUndefined();

      await expect(pxe.getExtendedContractData(goodDeploy.instance!.address)).resolves.toBeDefined();
      await expect(pxe.getExtendedContractData(goodDeploy.instance!.address)).resolves.toBeDefined();

      await expect(pxe.getContractData(badDeploy.instance!.address)).resolves.toBeUndefined();
      await expect(pxe.getExtendedContractData(badDeploy.instance!.address)).resolves.toBeUndefined();
    } finally {
      sequencer?.updateSequencerConfig({
        minTxsPerBlock: 1,
      });
    }
  }, 60_000);

  // Tests calling a private function in an uninitialized and undeployed contract. Note that
  // it still requires registering the contract artifact and instance locally in the pxe.
  test.each(['as entrypoint', 'from an account contract'] as const)(
    'executes a function in an undeployed contract %s',
    async kind => {
      const testWallet = kind === 'as entrypoint' ? new SignerlessWallet(pxe) : wallet;
      const contract = await registerContract(testWallet, TestContract);
      const receipt = await contract.methods.emit_nullifier(10).send().wait({ debug: true });
      const expected = siloNullifier(contract.address, new Fr(10));
      expect(receipt.debugInfo?.newNullifiers[1]).toEqual(expected);
    },
  );

  // Tests privately initializing an undeployed contract. Also requires pxe registration in advance.
  test.each(['as entrypoint', 'from an account contract'] as const)(
    'privately initializes an undeployed contract contract %s',
    async kind => {
      const testWallet = kind === 'as entrypoint' ? new SignerlessWallet(pxe) : wallet;
      const owner = await registerRandomAccount(pxe);
      const initArgs: StatefulContractCtorArgs = [owner, 42];
      const contract = await registerContract(testWallet, StatefulTestContract, initArgs);
      await contract.methods
        .constructor(...initArgs)
        .send()
        .wait();
      expect(await contract.methods.summed_values(owner).view()).toEqual(42n);
    },
  );

  // Tests privately initializing multiple undeployed contracts on the same tx through an account contract.
  it('initializes multiple undeployed contracts in a single tx', async () => {
    const owner = await registerRandomAccount(pxe);
    const initArgs: StatefulContractCtorArgs[] = [42, 52].map(value => [owner, value]);
    const contracts = await Promise.all(initArgs.map(args => registerContract(wallet, StatefulTestContract, args)));
    const calls = contracts.map((c, i) => c.methods.constructor(...initArgs[i]).request());
    await new BatchCall(wallet, calls).send().wait();
    expect(await contracts[0].methods.summed_values(owner).view()).toEqual(42n);
    expect(await contracts[1].methods.summed_values(owner).view()).toEqual(52n);
  });

  // Tests registering a new contract class on a node and then deploying an instance.
  // These tests look scary, but don't fret: all this hodgepodge of calls will be hidden
  // behind a much nicer API in the near future as part of #4080.
  describe('registering a contract class', () => {
    let artifact: ContractArtifact;
    let contractClass: ContractClassWithId & ContractClassIdPreimage;

    beforeAll(async () => {
      artifact = StatefulTestContract.artifact;
      await registerContractClass(wallet, artifact).send().wait();
      contractClass = getContractClassFromArtifact(artifact);
    }, 60_000);

    it('registers the contract class on the node', async () => {
      const registeredClass = await aztecNode.getContractClass(contractClass.id);
      expect(registeredClass).toBeDefined();
      expect(registeredClass!.artifactHash.toString()).toEqual(contractClass.artifactHash.toString());
      expect(registeredClass!.privateFunctionsRoot.toString()).toEqual(contractClass.privateFunctionsRoot.toString());
      expect(registeredClass!.packedBytecode.toString('hex')).toEqual(contractClass.packedBytecode.toString('hex'));
      expect(registeredClass!.publicFunctions).toEqual(contractClass.publicFunctions);
      expect(registeredClass!.privateFunctions).toEqual([]);
    });

    it('broadcasts a private function', async () => {
      const selector = contractClass.privateFunctions[0].selector;
      await broadcastPrivateFunction(wallet, artifact, selector).send().wait();
      // TODO(#4428): Test that these functions are captured by the node and made available when
      // requesting the corresponding contract class.
    }, 60_000);

    it('broadcasts an unconstrained function', async () => {
      const functionArtifact = artifact.functions.find(fn => fn.functionType === FunctionType.UNCONSTRAINED)!;
      const selector = FunctionSelector.fromNameAndParameters(functionArtifact);
      await broadcastUnconstrainedFunction(wallet, artifact, selector).send().wait();
      // TODO(#4428): Test that these functions are captured by the node and made available when
      // requesting the corresponding contract class.
    }, 60_000);

    describe('deploying a contract instance', () => {
      let instance: ContractInstanceWithAddress;
      let deployer: ContractInstanceDeployerContract;
      let deployTxHash: TxHash;
      let initArgs: StatefulContractCtorArgs;
      let publicKey: PublicKey;

      beforeAll(async () => {
        initArgs = [accounts[0].address, 42];
        deployer = await registerContract(wallet, ContractInstanceDeployerContract, [], { salt: new Fr(1) });

        const salt = Fr.random();
        const portalAddress = EthAddress.random();
        publicKey = Point.random();

        instance = getContractInstanceFromDeployParams(artifact, initArgs, salt, publicKey, portalAddress);
        const { address, contractClassId } = instance;
        logger(`Deploying contract instance at ${address.toString()} class id ${contractClassId.toString()}`);

        const tx = await deployInstance(wallet, instance).send().wait();
        deployTxHash = tx.txHash;
      });

      it('emits deployment log', async () => {
        const logs = await pxe.getUnencryptedLogs({ txHash: deployTxHash });
        const deployedLog = logs.logs[0].log;
        expect(deployedLog.contractAddress).toEqual(deployer.address);
      });

      it('stores contract instance in the aztec node', async () => {
        const deployed = await aztecNode.getContract(instance.address);
        expect(deployed).toBeDefined();
        expect(deployed!.address).toEqual(instance.address);
        expect(deployed!.contractClassId).toEqual(contractClass.id);
        expect(deployed!.initializationHash).toEqual(instance.initializationHash);
        expect(deployed!.portalContractAddress).toEqual(instance.portalContractAddress);
        expect(deployed!.publicKeysHash).toEqual(instance.publicKeysHash);
        expect(deployed!.salt).toEqual(instance.salt);
      });

      it('calls a public function on the deployed instance', async () => {
        // TODO(@spalladino) We should **not** need the whole instance, including initArgs and salt,
        // in order to interact with a public function for the contract. We may even not need
        // all of it for running a private function. Consider removing `instance` as a required
        // field in the aztec.js `Contract` class, maybe we can replace it with just the partialAddress.
        // Not just that, but this instance has been broadcasted, so the pxe should be able to get
        // its information from the node directly, excluding private functions, but it's ok because
        // we are not going to run those - but this may require registering "partial" contracts in the pxe.
        // Anyway, when we implement that, we should be able to replace this `registerContract` with
        // a simpler `Contract.at(instance.address, wallet)`.
        const registered = await registerContract(wallet, StatefulTestContract, initArgs, {
          salt: instance.salt,
          portalAddress: instance.portalContractAddress,
          publicKey,
        });
        expect(registered.address).toEqual(instance.address);
        const contract = await StatefulTestContract.at(instance.address, wallet);
        const whom = AztecAddress.random();
        await contract.methods.increment_public_value(whom, 10).send({ skipPublicSimulation: true }).wait();
        const stored = await contract.methods.get_public_value(whom).view();
        expect(stored).toEqual(10n);
      });
    });
  });
});

type StatefulContractCtorArgs = Parameters<StatefulTestContract['methods']['constructor']>;

async function registerRandomAccount(pxe: PXE): Promise<AztecAddress> {
  const { completeAddress: owner, privateKey } = CompleteAddress.fromRandomPrivateKey();
  await pxe.registerAccount(privateKey, owner.partialAddress);
  return owner.address;
}

type ContractArtifactClass<T extends ContractBase> = {
  at(address: AztecAddress, wallet: Wallet): Promise<T>;
  artifact: ContractArtifact;
};

async function registerContract<T extends ContractBase>(
  wallet: Wallet,
  contractArtifact: ContractArtifactClass<T>,
  args: any[] = [],
  opts: { salt?: Fr; publicKey?: Point; portalAddress?: EthAddress } = {},
): Promise<T> {
  const { salt, publicKey, portalAddress } = opts;
  const instance = getContractInstanceFromDeployParams(contractArtifact.artifact, args, salt, publicKey, portalAddress);
  await wallet.addContracts([{ artifact: contractArtifact.artifact, instance }]);
  return contractArtifact.at(instance.address, wallet);
}
