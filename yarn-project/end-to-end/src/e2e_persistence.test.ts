import { getUnsafeSchnorrAccount, getUnsafeSchnorrWallet } from '@aztec/accounts/single_key';
import { AccountWallet, waitForAccountSynch } from '@aztec/aztec.js';
import { CompleteAddress, EthAddress, Fq, Fr } from '@aztec/circuits.js';
import { DeployL1Contracts } from '@aztec/ethereum';
import { EasyPrivateTokenContract } from '@aztec/noir-contracts/EasyPrivateToken';

import { mkdtemp } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

import { EndToEndContext, setup } from './fixtures/utils.js';

describe('Aztec persistence', () => {
  /**
   * These tests check that the Aztec Node and PXE can be shutdown and restarted without losing data.
   *
   * There are four scenarios to check:
   * 1. Node and PXE are started with an existing databases
   * 2. PXE is started with an existing database and connects to a Node with an empty database
   * 3. PXE is started with an empty database and connects to a Node with an existing database
   * 4. PXE is started with an empty database and connects to a Node with an empty database
   *
   * All four scenarios use the same L1 state, which is deployed in the `beforeAll` hook.
   */

  // the test contract and account deploying it
  let contractAddress: CompleteAddress;
  let ownerPrivateKey: Fq;
  let ownerAddress: CompleteAddress;

  // a directory where data will be persisted by components
  // passing this through to the Node or PXE will control whether they use persisted data or not
  let dataDirectory: string;

  // state that is persisted between tests
  let deployL1ContractsValues: DeployL1Contracts;

  let context: EndToEndContext;

  // deploy L1 contracts, start initial node & PXE, deploy test contract & shutdown node and PXE
  beforeAll(async () => {
    dataDirectory = await mkdtemp(join(tmpdir(), 'aztec-node-'));

    const initialContext = await setup(0, { dataDirectory }, { dataDirectory });
    deployL1ContractsValues = initialContext.deployL1ContractsValues;

    ownerPrivateKey = Fq.random();
    const ownerWallet = await getUnsafeSchnorrAccount(initialContext.pxe, ownerPrivateKey, Fr.ZERO).waitDeploy();
    ownerAddress = ownerWallet.getCompleteAddress();

    const deployer = EasyPrivateTokenContract.deploy(ownerWallet, 1000n, ownerWallet.getAddress());
    await deployer.simulate({});

    const contract = await deployer.send().deployed();
    contractAddress = contract.completeAddress;

    await initialContext.teardown();
  }, 100_000);

  describe.each([
    [
      // ie we were shutdown and now starting back up. Initial sync should be ~instant
      'when starting Node and PXE with existing databases',
      () => setup(0, { dataDirectory, deployL1ContractsValues }, { dataDirectory }),
      1000,
    ],
    [
      // ie our PXE was restarted, data kept intact and now connects to a "new" Node. Initial synch will synch from scratch
      'when starting a PXE with an existing database, connected to a Node with database synched from scratch',
      () => setup(0, { deployL1ContractsValues }, { dataDirectory }),
      10_000,
    ],
  ])('%s', (_, contextSetup, timeout) => {
    let ownerWallet: AccountWallet;
    let contract: EasyPrivateTokenContract;

    beforeEach(async () => {
      context = await contextSetup();
      ownerWallet = await getUnsafeSchnorrWallet(context.pxe, ownerAddress.address, ownerPrivateKey);
      contract = await EasyPrivateTokenContract.at(contractAddress.address, ownerWallet);
    }, timeout);

    afterEach(async () => {
      await context.teardown();
    });

    it('correctly restores balances', async () => {
      // test for >0 instead of exact value so test isn't dependent on run order
      await expect(contract.methods.getBalance(ownerWallet.getAddress()).view()).resolves.toBeGreaterThan(0n);
    });

    it('tracks new notes for the owner', async () => {
      const balance = await contract.methods.getBalance(ownerWallet.getAddress()).view();
      await contract.methods.mint(1000n, ownerWallet.getAddress()).send().wait();
      await expect(contract.methods.getBalance(ownerWallet.getAddress()).view()).resolves.toEqual(balance + 1000n);
    });

    it('allows transfers of tokens from owner', async () => {
      const otherWallet = await getUnsafeSchnorrAccount(context.pxe, Fq.random(), Fr.ZERO).waitDeploy();

      const initialOwnerBalance = await contract.methods.getBalance(ownerWallet.getAddress()).view();
      await contract.methods.transfer(500n, ownerWallet.getAddress(), otherWallet.getAddress()).send().wait();
      const [ownerBalance, targetBalance] = await Promise.all([
        contract.methods.getBalance(ownerWallet.getAddress()).view(),
        contract.methods.getBalance(otherWallet.getAddress()).view(),
      ]);

      expect(ownerBalance).toEqual(initialOwnerBalance - 500n);
      expect(targetBalance).toEqual(500n);
    });
  });

  describe.each([
    [
      // ie. I'm setting up a new full node, sync from scratch and restore wallets/notes
      'when starting the Node and PXE with empty databases',
      () => setup(0, { deployL1ContractsValues }, {}),
      10_000,
    ],
    [
      // ie. I'm setting up a new PXE, restore wallets/notes from a Node
      'when starting a PXE with an empty database connected to a Node with an existing database',
      () => setup(0, { dataDirectory, deployL1ContractsValues }, {}),
      10_000,
    ],
  ])('%s', (_, contextSetup, timeout) => {
    beforeEach(async () => {
      context = await contextSetup();
    }, timeout);
    afterEach(async () => {
      await context.teardown();
    });

    it('pxe does not have the owner account', async () => {
      await expect(context.pxe.getRecipient(ownerAddress.address)).resolves.toBeUndefined();
    });

    it('the node has the contract', async () => {
      await expect(context.aztecNode.getContractData(contractAddress.address)).resolves.toBeDefined();
    });

    it('pxe does not know of the deployed contract', async () => {
      await context.pxe.registerRecipient(ownerAddress);

      const wallet = await getUnsafeSchnorrAccount(context.pxe, Fq.random(), Fr.ZERO).waitDeploy();
      const contract = await EasyPrivateTokenContract.at(contractAddress.address, wallet);
      await expect(contract.methods.getBalance(ownerAddress.address).view()).rejects.toThrowError(/Unknown contract/);
    });

    it("pxe does not have owner's notes", async () => {
      await context.pxe.addContracts([
        {
          artifact: EasyPrivateTokenContract.artifact,
          completeAddress: contractAddress,
          portalContract: EthAddress.ZERO,
        },
      ]);
      await context.pxe.registerRecipient(ownerAddress);

      const wallet = await getUnsafeSchnorrAccount(context.pxe, Fq.random(), Fr.ZERO).waitDeploy();
      const contract = await EasyPrivateTokenContract.at(contractAddress.address, wallet);
      await expect(contract.methods.getBalance(ownerAddress.address).view()).resolves.toEqual(0n);
    });

    it('pxe restores notes after registering the owner', async () => {
      await context.pxe.addContracts([
        {
          artifact: EasyPrivateTokenContract.artifact,
          completeAddress: contractAddress,
          portalContract: EthAddress.ZERO,
        },
      ]);

      await context.pxe.registerAccount(ownerPrivateKey, ownerAddress.partialAddress);
      const ownerWallet = await getUnsafeSchnorrAccount(context.pxe, ownerPrivateKey, ownerAddress).getWallet();
      const contract = await EasyPrivateTokenContract.at(contractAddress.address, ownerWallet);

      await waitForAccountSynch(context.pxe, ownerAddress, { interval: 1, timeout: 10 });

      // check that notes total more than 0 so that this test isn't dependent on run order
      await expect(contract.methods.getBalance(ownerAddress.address).view()).resolves.toBeGreaterThan(0n);
    });
  });
});
