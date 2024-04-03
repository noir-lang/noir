import {
  AztecAddress,
  type AztecNode,
  BatchCall,
  CompleteAddress,
  type ContractArtifact,
  type ContractBase,
  type ContractClassWithId,
  ContractDeployer,
  type ContractInstanceWithAddress,
  type DebugLogger,
  EthAddress,
  Fr,
  type PXE,
  SignerlessWallet,
  TxStatus,
  type Wallet,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
} from '@aztec/aztec.js';
import {
  broadcastPrivateFunction,
  broadcastUnconstrainedFunction,
  deployInstance,
  registerContractClass,
} from '@aztec/aztec.js/deployment';
import { type ContractClassIdPreimage, Point } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { writeTestData } from '@aztec/foundation/testing';
import { CounterContract, StatefulTestContract } from '@aztec/noir-contracts.js';
import { TestContract, TestContractArtifact } from '@aztec/noir-contracts.js/Test';
import { TokenContract, TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
import { type SequencerClient } from '@aztec/sequencer-client';

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
      const salt = Fr.random();
      const publicKey = accounts[0].publicKey;
      const deploymentData = getContractInstanceFromDeployParams(TestContractArtifact, {
        salt,
        publicKey,
        deployer: wallet.getAddress(),
      });
      const deployer = new ContractDeployer(TestContractArtifact, wallet, publicKey);
      const receipt = await deployer.deploy().send({ contractAddressSalt: salt }).wait({ wallet });
      expect(receipt.contract.address).toEqual(deploymentData.address);
      expect(await pxe.getContractInstance(deploymentData.address)).toBeDefined();
      expect(await pxe.isContractPubliclyDeployed(deploymentData.address)).toBeDefined();
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

      const expectedPortal = portalContract.toString();
      expect((await pxe.getContractInstance(address))?.portalContractAddress.toString()).toEqual(expectedPortal);
    }, 60_000);

    it('should not deploy a contract which failed the public part of the execution', async () => {
      sequencer?.updateSequencerConfig({ minTxsPerBlock: 2 });
      try {
        // This test requires at least another good transaction to go through in the same block as the bad one.
        const artifact = TokenContractArtifact;
        const initArgs = ['TokenName', 'TKN', 18] as const;
        const goodDeploy = StatefulTestContract.deploy(wallet, accounts[0], 42);
        const badDeploy = new ContractDeployer(artifact, wallet).deploy(AztecAddress.ZERO, ...initArgs);

        const firstOpts = { skipPublicSimulation: true, skipClassRegistration: true, skipInstanceDeploy: true };
        const secondOpts = { skipPublicSimulation: true };

        await Promise.all([goodDeploy.prove(firstOpts), badDeploy.prove(secondOpts)]);
        const [goodTx, badTx] = [goodDeploy.send(firstOpts), badDeploy.send(secondOpts)];
        const [goodTxPromiseResult, badTxReceiptResult] = await Promise.allSettled([
          goodTx.wait(),
          badTx.wait({ dontThrowOnRevert: true }),
        ]);

        expect(goodTxPromiseResult.status).toBe('fulfilled');
        expect(badTxReceiptResult.status).toBe('fulfilled'); // but reverted

        const [goodTxReceipt, badTxReceipt] = await Promise.all([goodTx.getReceipt(), badTx.getReceipt()]);

        // Both the good and bad transactions are included
        expect(goodTxReceipt.blockNumber).toEqual(expect.any(Number));
        expect(badTxReceipt.blockNumber).toEqual(expect.any(Number));

        expect(badTxReceipt.status).toEqual(TxStatus.REVERTED);

        // But the bad tx did not deploy
        await expect(pxe.isContractClassPubliclyRegistered(badDeploy.getInstance().address)).resolves.toBeFalsy();
      } finally {
        sequencer?.updateSequencerConfig({ minTxsPerBlock: 1 });
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
        expect(receipt.debugInfo?.nullifiers[1]).toEqual(expected);
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
        const contract = await registerContract(testWallet, StatefulTestContract, { initArgs });
        logger.info(`Calling the constructor for ${contract.address}`);
        await contract.methods
          .constructor(...initArgs)
          .send()
          .wait();
        logger.info(`Checking if the constructor was run for ${contract.address}`);
        expect(await contract.methods.summed_values(owner).simulate()).toEqual(42n);
        logger.info(`Calling a private function that requires initialization on ${contract.address}`);
        await contract.methods.create_note(owner, 10).send().wait();
        expect(await contract.methods.summed_values(owner).simulate()).toEqual(52n);
      },
      30_000,
    );

    // Tests privately initializing multiple undeployed contracts on the same tx through an account contract.
    it('initializes multiple undeployed contracts in a single tx', async () => {
      const owner = await registerRandomAccount(pxe);
      const initArgss: StatefulContractCtorArgs[] = [42, 52].map(value => [owner, value]);
      const contracts = await Promise.all(
        initArgss.map(initArgs => registerContract(wallet, StatefulTestContract, { initArgs })),
      );
      const calls = contracts.map((c, i) => c.methods.constructor(...initArgss[i]).request());
      await new BatchCall(wallet, calls).send().wait();
      expect(await contracts[0].methods.summed_values(owner).simulate()).toEqual(42n);
      expect(await contracts[1].methods.summed_values(owner).simulate()).toEqual(52n);
    }, 30_000);

    // TODO(@spalladino): This won't work until we can read a nullifier in the same tx in which it was emitted.
    it.skip('initializes and calls a private function in a single tx', async () => {
      const owner = await registerRandomAccount(pxe);
      const initArgs: StatefulContractCtorArgs = [owner, 42];
      const contract = await registerContract(wallet, StatefulTestContract, { initArgs });
      const batch = new BatchCall(wallet, [
        contract.methods.constructor(...initArgs).request(),
        contract.methods.create_note(owner, 10).request(),
      ]);
      logger.info(`Executing constructor and private function in batch at ${contract.address}`);
      await batch.send().wait();
      expect(await contract.methods.summed_values(owner).simulate()).toEqual(52n);
    });

    it('refuses to initialize a contract twice', async () => {
      const owner = await registerRandomAccount(pxe);
      const initArgs: StatefulContractCtorArgs = [owner, 42];
      const contract = await registerContract(wallet, StatefulTestContract, { initArgs });
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
      const contract = await registerContract(wallet, StatefulTestContract, { initArgs });
      // TODO(@spalladino): It'd be nicer to be able to fail the assert with a more descriptive message.
      await expect(contract.methods.create_note(owner, 10).send().wait()).rejects.toThrow(
        /nullifier witness not found/i,
      );
    });

    it('refuses to initialize a contract with incorrect args', async () => {
      const owner = await registerRandomAccount(pxe);
      const contract = await registerContract(wallet, StatefulTestContract, { initArgs: [owner, 42] });
      await expect(contract.methods.constructor(owner, 43).prove()).rejects.toThrow(
        /Initialization hash does not match/,
      );
    });

    it('refuses to initialize an instance from a different deployer', async () => {
      const owner = await registerRandomAccount(pxe);
      const contract = await registerContract(wallet, StatefulTestContract, { initArgs: [owner, 42], deployer: owner });
      await expect(contract.methods.constructor(owner, 42).prove()).rejects.toThrow(
        /Initializer address is not the contract deployer/i,
      );
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
      const tx = await broadcastPrivateFunction(wallet, artifact, selector).send().wait();
      const logs = await pxe.getUnencryptedLogs({ txHash: tx.txHash });
      const logData = logs.logs[0].log.data;
      writeTestData('yarn-project/circuits.js/fixtures/PrivateFunctionBroadcastedEventData.hex', logData);

      const fetchedClass = await aztecNode.getContractClass(contractClass.id);
      const fetchedFunction = fetchedClass!.privateFunctions[0]!;
      expect(fetchedFunction).toBeDefined();
      expect(fetchedFunction.selector).toEqual(selector);
    }, 60_000);

    it('broadcasts an unconstrained function', async () => {
      const functionArtifact = artifact.functions.find(fn => fn.functionType === FunctionType.UNCONSTRAINED)!;
      const selector = FunctionSelector.fromNameAndParameters(functionArtifact);
      const tx = await broadcastUnconstrainedFunction(wallet, artifact, selector).send().wait();
      const logs = await pxe.getUnencryptedLogs({ txHash: tx.txHash });
      const logData = logs.logs[0].log.data;
      writeTestData('yarn-project/circuits.js/fixtures/UnconstrainedFunctionBroadcastedEventData.hex', logData);

      const fetchedClass = await aztecNode.getContractClass(contractClass.id);
      const fetchedFunction = fetchedClass!.unconstrainedFunctions[0]!;
      expect(fetchedFunction).toBeDefined();
      expect(fetchedFunction.selector).toEqual(selector);
    }, 60_000);

    const testDeployingAnInstance = (how: string, deployFn: (toDeploy: ContractInstanceWithAddress) => Promise<void>) =>
      describe(`deploying a contract instance ${how}`, () => {
        let instance: ContractInstanceWithAddress;
        let initArgs: StatefulContractCtorArgs;
        let contract: StatefulTestContract;

        const deployInstance = async (opts: { constructorName?: string; deployer?: AztecAddress } = {}) => {
          const initArgs = [accounts[0].address, 42] as StatefulContractCtorArgs;
          const salt = Fr.random();
          const portalAddress = EthAddress.random();
          const publicKey = Point.random();
          const instance = getContractInstanceFromDeployParams(artifact, {
            constructorArgs: initArgs,
            salt,
            publicKey,
            portalAddress,
            constructorArtifact: opts.constructorName,
            deployer: opts.deployer,
          });
          const { address, contractClassId } = instance;
          logger(`Deploying contract instance at ${address.toString()} class id ${contractClassId.toString()}`);
          await deployFn(instance);

          // TODO(@spalladino) We should **not** need the whole instance, including initArgs and salt,
          // in order to interact with a public function for the contract. We may even not need
          // all of it for running a private function. Consider removing `instance` as a required
          // field in the aztec.js `Contract` class, maybe we can replace it with just the partialAddress.
          // Not just that, but this instance has been broadcasted, so the pxe should be able to get
          // its information from the node directly, excluding private functions, but it's ok because
          // we are not going to run those - but this may require registering "partial" contracts in the pxe.
          // Anyway, when we implement that, we should be able to replace this `registerContract` with
          // a simpler `Contract.at(instance.address, wallet)`.
          const registered = await registerContract(wallet, StatefulTestContract, {
            constructorName: opts.constructorName,
            salt: instance.salt,
            portalAddress: instance.portalContractAddress,
            publicKey,
            initArgs,
            deployer: opts.deployer,
          });
          expect(registered.address).toEqual(instance.address);
          const contract = await StatefulTestContract.at(instance.address, wallet);
          return { contract, initArgs, instance, publicKey };
        };

        describe('using a private constructor', () => {
          beforeAll(async () => {
            ({ instance, initArgs, contract } = await deployInstance());
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
            expect(deployed!.deployer).toEqual(instance.deployer);
          });

          it('calls a public function with no init check on the deployed instance', async () => {
            const whom = AztecAddress.random();
            await contract.methods
              .increment_public_value_no_init_check(whom, 10)
              .send({ skipPublicSimulation: true })
              .wait();
            const stored = await contract.methods.get_public_value(whom).simulate();
            expect(stored).toEqual(10n);
          }, 30_000);

          it('refuses to call a public function with init check if the instance is not initialized', async () => {
            const whom = AztecAddress.random();
            const receipt = await contract.methods
              .increment_public_value(whom, 10)
              .send({ skipPublicSimulation: true })
              .wait({ dontThrowOnRevert: true });
            expect(receipt.status).toEqual(TxStatus.REVERTED);

            // Meanwhile we check we didn't increment the value
            expect(await contract.methods.get_public_value(whom).simulate()).toEqual(0n);
          }, 30_000);

          it('refuses to initialize the instance with wrong args via a private function', async () => {
            await expect(contract.methods.constructor(AztecAddress.random(), 43).prove()).rejects.toThrow(
              /initialization hash does not match/i,
            );
          }, 30_000);

          it('initializes the contract and calls a public function', async () => {
            await contract.methods
              .constructor(...initArgs)
              .send()
              .wait();
            const whom = AztecAddress.random();
            await contract.methods.increment_public_value(whom, 10).send({ skipPublicSimulation: true }).wait();
            const stored = await contract.methods.get_public_value(whom).simulate();
            expect(stored).toEqual(10n);
          }, 30_000);

          it('refuses to reinitialize the contract', async () => {
            await expect(
              contract.methods
                .constructor(...initArgs)
                .send({ skipPublicSimulation: true })
                .wait(),
            ).rejects.toThrow(/dropped/i);
          }, 30_000);
        });

        describe('using a public constructor', () => {
          beforeAll(async () => {
            ({ instance, initArgs, contract } = await deployInstance({ constructorName: 'public_constructor' }));
          }, 60_000);

          it('refuses to initialize the instance with wrong args via a public function', async () => {
            const whom = AztecAddress.random();
            const receipt = await contract.methods
              .public_constructor(whom, 43)
              .send({ skipPublicSimulation: true })
              .wait({ dontThrowOnRevert: true });
            expect(receipt.status).toEqual(TxStatus.REVERTED);
            expect(await contract.methods.get_public_value(whom).simulate()).toEqual(0n);
          }, 30_000);

          it('initializes the contract and calls a public function', async () => {
            await contract.methods
              .public_constructor(...initArgs)
              .send()
              .wait();
            const whom = AztecAddress.random();
            await contract.methods.increment_public_value(whom, 10).send({ skipPublicSimulation: true }).wait();
            const stored = await contract.methods.get_public_value(whom).simulate();
            expect(stored).toEqual(10n);
          }, 30_000);

          it('refuses to reinitialize the contract', async () => {
            await expect(
              contract.methods
                .public_constructor(...initArgs)
                .send({ skipPublicSimulation: true })
                .wait(),
            ).rejects.toThrow(/dropped/i);
          }, 30_000);
        });
      });

    testDeployingAnInstance('from a wallet', async instance => {
      // Calls the deployer contract directly from a wallet
      await deployInstance(wallet, instance).send().wait();
    });

    testDeployingAnInstance('from a contract', async instance => {
      // Register the instance to be deployed in the pxe
      await wallet.registerContract({ artifact, instance });
      // Set up the contract that calls the deployer (which happens to be the TestContract) and call it
      const deployer = await TestContract.deploy(wallet).send().deployed();
      await deployer.methods.deploy_contract(instance.address).send().wait();
    });

    describe('error scenarios in deployment', () => {
      it('refuses to call a public function on an undeployed contract', async () => {
        const whom = accounts[0].address;
        const instance = await registerContract(wallet, StatefulTestContract, { initArgs: [whom, 42] });
        await expect(
          instance.methods.increment_public_value_no_init_check(whom, 10).send({ skipPublicSimulation: true }).wait(),
        ).rejects.toThrow(/dropped/);
      });

      it('refuses to deploy an instance from a different deployer', () => {
        const instance = getContractInstanceFromDeployParams(artifact, {
          constructorArgs: [AztecAddress.random(), 42],
          deployer: AztecAddress.random(),
        });
        expect(() => deployInstance(wallet, instance)).toThrow(/does not match/i);
      });
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
      logger.debug(`Deploying stateful test contract`);
      const contract = await StatefulTestContract.deploy(wallet, owner, 42).send().deployed();
      expect(await contract.methods.summed_values(owner).simulate()).toEqual(42n);
      logger.debug(`Calling public method on stateful test contract at ${contract.address.toString()}`);
      await contract.methods.increment_public_value(owner, 84).send().wait();
      expect(await contract.methods.get_public_value(owner).simulate()).toEqual(84n);
    }, 60_000);

    it('publicly universally deploys and initializes a contract', async () => {
      const owner = accounts[0];
      const opts = { universalDeploy: true };
      const contract = await StatefulTestContract.deploy(wallet, owner, 42).send(opts).deployed();
      expect(await contract.methods.summed_values(owner).simulate()).toEqual(42n);
      await contract.methods.increment_public_value(owner, 84).send().wait();
      expect(await contract.methods.get_public_value(owner).simulate()).toEqual(84n);
    }, 60_000);

    it('publicly deploys and calls a public function from the constructor', async () => {
      const owner = accounts[0];
      const token = await TokenContract.deploy(wallet, owner, 'TOKEN', 'TKN', 18).send().deployed();
      expect(await token.methods.is_minter(owner).simulate()).toEqual(true);
    }, 60_000);

    it('publicly deploys and initializes via a public function', async () => {
      const owner = accounts[0];
      logger.debug(`Deploying contract via a public constructor`);
      const contract = await StatefulTestContract.deployWithOpts({ wallet, method: 'public_constructor' }, owner, 42)
        .send()
        .deployed();
      expect(await contract.methods.get_public_value(owner).simulate()).toEqual(42n);
      logger.debug(`Calling a private function to ensure the contract was properly initialized`);
      await contract.methods.create_note(owner, 30).send().wait();
      expect(await contract.methods.summed_values(owner).simulate()).toEqual(30n);
    }, 60_000);

    it('deploys a contract with a default initializer not named constructor', async () => {
      logger.debug(`Deploying contract with a default initializer named initialize`);
      const opts = { skipClassRegistration: true, skipPublicDeployment: true };
      const contract = await CounterContract.deploy(wallet, 10, accounts[0]).send(opts).deployed();
      logger.debug(`Calling a function to ensure the contract was properly initialized`);
      await contract.methods.increment(accounts[0]).send().wait();
      expect(await contract.methods.get_counter(accounts[0]).simulate()).toEqual(11n);
    });

    it('publicly deploys a contract with no constructor', async () => {
      logger.debug(`Deploying contract with no constructor`);
      const contract = await TestContract.deploy(wallet).send().deployed();
      logger.debug(`Call a public function to check that it was publicly deployed`);
      const receipt = await contract.methods.emit_unencrypted(42).send().wait();
      const logs = await pxe.getUnencryptedLogs({ txHash: receipt.txHash });
      expect(logs.logs[0].log.data.toString('hex').replace(/^0+/, '')).toEqual('2a');
    });

    it('refuses to deploy a contract with no constructor and no public deployment', async () => {
      logger.debug(`Deploying contract with no constructor and skipping public deploy`);
      const opts = { skipPublicDeployment: true, skipClassRegistration: true };
      await expect(TestContract.deploy(wallet).prove(opts)).rejects.toThrow(/no function calls needed/i);
    });

    it.skip('publicly deploys and calls a public function in the same batched call', async () => {
      // TODO(@spalladino): Requires being able to read a nullifier on the same tx it was emitted.
    });

    it.skip('publicly deploys and calls a public function in a tx in the same block', async () => {
      // TODO(@spalladino): Requires being able to read a nullifier on the same block it was emitted.
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
  opts: {
    salt?: Fr;
    publicKey?: Point;
    portalAddress?: EthAddress;
    initArgs?: any[];
    constructorName?: string;
    deployer?: AztecAddress;
  } = {},
): Promise<T> {
  const { salt, publicKey, portalAddress, initArgs, constructorName, deployer } = opts;
  const instance = getContractInstanceFromDeployParams(contractArtifact.artifact, {
    constructorArgs: initArgs ?? [],
    constructorArtifact: constructorName,
    salt,
    publicKey,
    portalAddress,
    deployer,
  });
  await wallet.registerContract({ artifact: contractArtifact.artifact, instance });
  return contractArtifact.at(instance.address, wallet);
}
