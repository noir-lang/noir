import { AztecAddress, CompleteAddress, Fr, FunctionData, TxContext } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ConstantKeyPair } from '@aztec/key-store';
import {
  AztecRPC,
  DeployedContract,
  INITIAL_L2_BLOCK_NUM,
  TxExecutionRequest,
  randomDeployedContract,
} from '@aztec/types';

export const aztecRpcTestSuite = (testName: string, aztecRpcSetup: () => Promise<AztecRPC>) => {
  describe(testName, function () {
    let rpc: AztecRPC;

    beforeAll(async () => {
      rpc = await aztecRpcSetup();
    }, 120_000);

    it('registers an account and returns it as an account only and not as a recipient', async () => {
      const keyPair = ConstantKeyPair.random(await Grumpkin.new());
      const completeAddress = await CompleteAddress.fromPrivateKey(await keyPair.getPrivateKey());

      await rpc.registerAccount(await keyPair.getPrivateKey(), completeAddress);

      // Check that the account is correctly registered using the getAccounts and getRecipients methods
      const accounts = await rpc.getAccounts();
      const recipients = await rpc.getRecipients();
      expect(accounts).toContainEqual(completeAddress);
      expect(recipients).not.toContainEqual(completeAddress);

      // Check that the account is correctly registered using the getAccount and getRecipient methods
      const account = await rpc.getAccount(completeAddress.address);
      const recipient = await rpc.getRecipient(completeAddress.address);
      expect(account).toEqual(completeAddress);
      expect(recipient).toBeUndefined();
    });

    it('registers a recipient and returns it as a recipient only and not as an account', async () => {
      const completeAddress = await CompleteAddress.random();

      await rpc.registerRecipient(completeAddress);

      // Check that the recipient is correctly registered using the getAccounts and getRecipients methods
      const accounts = await rpc.getAccounts();
      const recipients = await rpc.getRecipients();
      expect(accounts).not.toContainEqual(completeAddress);
      expect(recipients).toContainEqual(completeAddress);

      // Check that the recipient is correctly registered using the getAccount and getRecipient methods
      const account = await rpc.getAccount(completeAddress.address);
      const recipient = await rpc.getRecipient(completeAddress.address);
      expect(account).toBeUndefined();
      expect(recipient).toEqual(completeAddress);
    });

    it('cannot register the same account twice', async () => {
      const keyPair = ConstantKeyPair.random(await Grumpkin.new());
      const completeAddress = await CompleteAddress.fromPrivateKey(await keyPair.getPrivateKey());

      await rpc.registerAccount(await keyPair.getPrivateKey(), completeAddress);
      await expect(async () => rpc.registerAccount(await keyPair.getPrivateKey(), completeAddress)).rejects.toThrow(
        `Complete address corresponding to ${completeAddress.address} already exists`,
      );
    });

    it('cannot register the same recipient twice', async () => {
      const completeAddress = await CompleteAddress.random();

      await rpc.registerRecipient(completeAddress);
      await expect(() => rpc.registerRecipient(completeAddress)).rejects.toThrow(
        `Complete address corresponding to ${completeAddress.address} already exists`,
      );
    });

    it('successfully adds a contract', async () => {
      const contracts: DeployedContract[] = [randomDeployedContract(), randomDeployedContract()];
      await rpc.addContracts(contracts);

      const expectedContractAddresses = contracts.map(contract => contract.address);
      const contractAddresses = await rpc.getContracts();

      // check if all the contracts were returned
      expect(contractAddresses).toEqual(expect.arrayContaining(expectedContractAddresses));
    });

    it('throws when simulating a tx targeting public entrypoint', async () => {
      const functionData = FunctionData.empty();
      functionData.isPrivate = false;
      const txExecutionRequest = new TxExecutionRequest(
        AztecAddress.random(),
        functionData,
        new Fr(0),
        TxContext.empty(),
        [],
      );

      await expect(async () => await rpc.simulateTx(txExecutionRequest)).rejects.toThrow(
        'Public entrypoints are not allowed',
      );
    });

    // Note: Not testing a successful run of `simulateTx`, `sendTx`, `getTxReceipt` and `viewTx` here as it requires
    //       a larger setup and it's sufficiently tested in the e2e tests.

    it('throws when getting public storage for non-existent contract', async () => {
      const contract = AztecAddress.random();
      await expect(async () => await rpc.getPublicStorageAt(contract, new Fr(0n))).rejects.toThrow(
        `Contract ${contract.toString()} is not deployed`,
      );
    });

    // Note: Not testing `getContractDataAndBytecode`, `getContractData` and `getUnencryptedLogs` here as these
    //       functions only call AztecNode and these methods are frequently used by the e2e tests.

    it('successfully gets a block number', async () => {
      const blockNum = await rpc.getBlockNum();
      expect(blockNum).toBeGreaterThanOrEqual(INITIAL_L2_BLOCK_NUM);
    });

    it('successfully gets node info', async () => {
      const nodeInfo = await rpc.getNodeInfo();
      expect(nodeInfo.version).toBeDefined();
      expect(nodeInfo.chainId).toBeDefined();
      expect(nodeInfo.rollupAddress).toBeDefined();
    });

    // Note: Not testing `isGlobalStateSynchronised`, `isAccountStateSynchronised` and `getSyncStatus` as these methods
    //       only call synchroniser.
  });
};
