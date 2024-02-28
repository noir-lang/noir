import {
  AztecAddress,
  AztecNode,
  BatchCall,
  CompleteAddress,
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
import { StatefulTestContract } from '@aztec/noir-contracts.js';
import { TestContract, TestContractArtifact } from '@aztec/noir-contracts.js/Test';
import { TokenContract, TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
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

  describe('legacy tests', () => {
    beforeAll(async () => {
      ({ teardown, pxe, accounts, logger, wallet, sequencer, aztecNode } = await setup());
    }, 100_000);

    afterAll(() => teardown());

    /**
     * Milestone 1.1.
     * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
     */
    it('should deploy a test contract', async () => {
      const publicKey = accounts[0].publicKey;
      const salt = Fr.random();
      const deploymentData = getContractInstanceFromDeployParams(
        TestContractArtifact,
        [],
        salt,
        publicKey,
        EthAddress.ZERO,
      );
      const deployer = new ContractDeployer(TestContractArtifact, wallet, publicKey);
      const receipt = await deployer.deploy().send({ contractAddressSalt: salt }).wait({ wallet });
      expect(receipt.contract.address).toEqual(deploymentData.address);
      expect(await isContractDeployed(pxe, deploymentData.address)).toBe(true);
      expect(await isContractDeployed(pxe, AztecAddress.random())).toBe(false);
    }, 60_000);

    /**
     * Verify that we can produce multiple rollups.
     */
    it('should deploy one contract after another in consecutive rollups', async () => {
      const deployer = new ContractDeployer(TestContractArtifact, wallet);

      for (let index = 0; index < 2; index++) {
        logger(`Deploying contract ${index + 1}...`);
        await deployer.deploy().send({ contractAddressSalt: Fr.random() }).wait({ wallet });
      }
    }, 60_000);

    /**
     * Verify that we can deploy multiple contracts and interact with all of them.
     */
    it('should deploy multiple contracts and interact with them', async () => {
      const deployer = new ContractDeployer(TestContractArtifact, wallet);

      for (let index = 0; index < 2; index++) {
        logger(`Deploying contract ${index + 1}...`);
        const receipt = await deployer.deploy().send({ contractAddressSalt: Fr.random() }).wait({ wallet });
        logger(`Sending TX to contract ${index + 1}...`);
        await receipt.contract.methods.get_public_key(accounts[0].address).send().wait();
      }
    }, 90_000);

    /**
     * Milestone 1.2.
     * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
     */
    it('should not deploy a contract with the same salt twice', async () => {
      const contractAddressSalt = Fr.random();
      const deployer = new ContractDeployer(TestContractArtifact, wallet);

      await deployer.deploy().send({ contractAddressSalt }).wait({ wallet });
      await expect(deployer.deploy().send({ contractAddressSalt }).wait()).rejects.toThrow(/dropped/);
    }, 60_000);

    it('should deploy a contract connected to a portal contract', async () => {
      const deployer = new ContractDeployer(TestContractArtifact, wallet);
      const portalContract = EthAddress.random();

      // ContractDeployer was instantiated with wallet so we don't have to pass it to wait(...)
      const receipt = await deployer.deploy().send({ portalContract }).wait();
      const address = receipt.contract.address;

      expect((await pxe.getContractData(address))?.portalContractAddress.toString()).toEqual(portalContract.toString());
      expect((await pxe.getExtendedContractData(address))?.contractData.portalContractAddress.toString()).toEqual(
        portalContract.toString(),
      );
    }, 60_000);

    // TODO(@spalladino): Review this test, it's showing an unexpected 'Bytecode not found' error in logs.
    // It's possible it is failing for the wrong reason, and the getContractData checks are returning wrong data.
    it('should not deploy a contract which failed the public part of the execution', async () => {
      sequencer?.updateSequencerConfig({
        minTxsPerBlock: 2,
      });

      try {
        // This test requires at least another good transaction to go through in the same block as the bad one.
        // I deployed the same contract again but it could really be any valid transaction here.
        const artifact = TokenContractArtifact;
        const initArgs = ['TokenName', 'TKN', 18] as const;
        const goodDeploy = new ContractDeployer(artifact, wallet).deploy(AztecAddress.random(), ...initArgs);
        const badDeploy = new ContractDeployer(artifact, wallet).deploy(AztecAddress.ZERO, ...initArgs);

        const firstOpts = { skipPublicSimulation: true };
        const secondOpts = { skipPublicSimulation: true, skipClassRegistration: true };

        await Promise.all([goodDeploy.simulate(firstOpts), badDeploy.simulate(secondOpts)]);
        const [goodTx, badTx] = [goodDeploy.send(firstOpts), badDeploy.send(secondOpts)];
        const [goodTxPromiseResult, badTxReceiptResult] = await Promise.allSettled([goodTx.wait(), badTx.wait()]);

        expect(goodTxPromiseResult.status).toBe('fulfilled');
        expect(badTxReceiptResult.status).toBe('rejected');

        const [goodTxReceipt, badTxReceipt] = await Promise.all([goodTx.getReceipt(), badTx.getReceipt()]);

        expect(goodTxReceipt.blockNumber).toEqual(expect.any(Number));
        expect(badTxReceipt.blockNumber).toBeUndefined();

        await expect(pxe.getContractData(goodDeploy.getInstance().address)).resolves.toBeDefined();
        await expect(pxe.getExtendedContractData(goodDeploy.getInstance().address)).resolves.toBeDefined();

        await expect(pxe.getContractData(badDeploy.getInstance().address)).resolves.toBeUndefined();
        await expect(pxe.getExtendedContractData(badDeploy.getInstance().address)).resolves.toBeUndefined();
      } finally {
        sequencer?.updateSequencerConfig({
          minTxsPerBlock: 1,
        });
      }
    }, 90_000);
  });

  describe('private initialization', () => {
    beforeAll(async () => {
      ({ teardown, pxe, accounts, logger, wallet, sequencer, aztecNode } = await setup());
    }, 100_000);
    afterAll(() => teardown());

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
      30_000,
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
      30_000,
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
    }, 30_000);

    it('refuses to initialize a contract twice', async () => {
      const owner = await registerRandomAccount(pxe);
      const initArgs: StatefulContractCtorArgs = [owner, 42];
      const contract = await registerContract(wallet, StatefulTestContract, initArgs);
      await contract.methods
        .constructor(...initArgs)
        .send()
        .wait();
      await expect(
        contract.methods
          .constructor(...initArgs)
          .send()
          .wait(),
      ).rejects.toThrow(/dropped/);
    });

    it('refuses to call a private function that requires initialization', async () => {
      const owner = await registerRandomAccount(pxe);
      const initArgs: StatefulContractCtorArgs = [owner, 42];
      const contract = await registerContract(wallet, StatefulTestContract, initArgs);
      // TODO(@spalladino): It'd be nicer to be able to fail the assert with a more descriptive message,
      // but the best we can do for now is pushing a read request to the kernel and wait for it to fail.
      // Maybe we need an unconstrained check for the read request that runs within the app circuit simulation
      // so we can bail earlier with a more descriptive error? I should create an issue for this.
      await expect(contract.methods.create_note(owner, 10).send().wait()).rejects.toThrow(
        /The read request.*does not match/,
      );
    });

    it('refuses to call a public function that requires initialization', async () => {
      // TODO(@spalladino)
    });
  });

  describe('registering a contract class', () => {
    beforeAll(async () => {
      ({ teardown, pxe, accounts, logger, wallet, sequencer, aztecNode } = await setup());
    }, 100_000);
    afterAll(() => teardown());

    let artifact: ContractArtifact;
    let contractClass: ContractClassWithId & ContractClassIdPreimage;

    beforeAll(async () => {
      artifact = StatefulTestContract.artifact;
      await registerContractClass(wallet, artifact).then(c => c.send().wait());
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

    const testDeployingAnInstance = (how: string, deployFn: (toDeploy: ContractInstanceWithAddress) => Promise<void>) =>
      describe(`deploying a contract instance ${how}`, () => {
        let instance: ContractInstanceWithAddress;
        let initArgs: StatefulContractCtorArgs;
        let publicKey: PublicKey;

        beforeAll(async () => {
          initArgs = [accounts[0].address, 42];
          const salt = Fr.random();
          const portalAddress = EthAddress.random();
          publicKey = Point.random();

          instance = getContractInstanceFromDeployParams(artifact, initArgs, salt, publicKey, portalAddress);
          const { address, contractClassId } = instance;
          logger(`Deploying contract instance at ${address.toString()} class id ${contractClassId.toString()}`);
          await deployFn(instance);
        }, 60_000);

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
        }, 30_000);
      });

    testDeployingAnInstance('from a wallet', async instance => {
      // Calls the deployer contract directly from a wallet
      await deployInstance(wallet, instance).send().wait();
    });

    testDeployingAnInstance('from a contract', async instance => {
      // Register the instance to be deployed in the pxe
      await wallet.addContracts([{ artifact, instance }]);
      // Set up the contract that calls the deployer (which happens to be the StatefulTestContract) and call it
      const deployer = await registerContract(wallet, StatefulTestContract, [accounts[0].address, 48]);
      await deployer.methods.deploy_contract(instance.address).send().wait();
    });
  });

  describe('using the contract deploy method', () => {
    // We use a beforeEach hook so we get a fresh pxe and node, so class registrations
    // from one test don't influence the others.
    // TODO(@spalladino): The above is only true for locally run e2e tests, on the CI this runs
    // on a single sandbox instance, so tests are not truly independent.
    beforeEach(async () => {
      ({ teardown, pxe, accounts, logger, wallet, sequencer, aztecNode } = await setup());
    }, 100_000);
    afterEach(() => teardown());

    it('publicly deploys and initializes a contract', async () => {
      const owner = accounts[0];
      const contract = await StatefulTestContract.deploy(wallet, owner, 42).send().deployed();
      expect(await contract.methods.summed_values(owner).view()).toEqual(42n);
      await contract.methods.increment_public_value(owner, 84).send().wait();
      expect(await contract.methods.get_public_value(owner).view()).toEqual(84n);
    }, 60_000);

    it('publicly deploys and calls a public function from the constructor', async () => {
      const owner = accounts[0];
      const token = await TokenContract.deploy(wallet, owner, 'TOKEN', 'TKN', 18).send().deployed();
      expect(await token.methods.is_minter(owner).view()).toEqual(true);
    }, 60_000);

    it.skip('publicly deploys and calls a public function in the same batched call', async () => {
      // TODO(@spalladino)
    });

    it.skip('publicly deploys and calls a public function in a tx in the same block', async () => {
      // TODO(@spalladino)
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
