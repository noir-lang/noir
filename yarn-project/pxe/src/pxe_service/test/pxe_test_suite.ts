import { AztecAddress, CompleteAddress, Fr, FunctionData, Point, TxContext } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ConstantKeyPair } from '@aztec/key-store';
import { DeployedContract, INITIAL_L2_BLOCK_NUM, PXE, TxExecutionRequest, randomDeployedContract } from '@aztec/types';

export const pxeTestSuite = (testName: string, pxeSetup: () => Promise<PXE>) => {
  describe(testName, () => {
    let pxe: PXE;

    beforeAll(async () => {
      pxe = await pxeSetup();
    }, 120_000);

    it('registers an account and returns it as an account only and not as a recipient', async () => {
      const keyPair = ConstantKeyPair.random(await Grumpkin.new());
      const completeAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(
        await keyPair.getPrivateKey(),
        Fr.random(),
      );

      await pxe.registerAccount(await keyPair.getPrivateKey(), completeAddress.partialAddress);

      // Check that the account is correctly registered using the getAccounts and getRecipients methods
      const accounts = await pxe.getRegisteredAccounts();
      const recipients = await pxe.getRecipients();
      expect(accounts).toContainEqual(completeAddress);
      expect(recipients).not.toContainEqual(completeAddress);

      // Check that the account is correctly registered using the getAccount and getRecipient methods
      const account = await pxe.getRegisteredAccount(completeAddress.address);
      const recipient = await pxe.getRecipient(completeAddress.address);
      expect(account).toEqual(completeAddress);
      expect(recipient).toBeUndefined();
    });

    it('registers a recipient and returns it as a recipient only and not as an account', async () => {
      const completeAddress = await CompleteAddress.random();

      await pxe.registerRecipient(completeAddress);

      // Check that the recipient is correctly registered using the getAccounts and getRecipients methods
      const accounts = await pxe.getRegisteredAccounts();
      const recipients = await pxe.getRecipients();
      expect(accounts).not.toContainEqual(completeAddress);
      expect(recipients).toContainEqual(completeAddress);

      // Check that the recipient is correctly registered using the getAccount and getRecipient methods
      const account = await pxe.getRegisteredAccount(completeAddress.address);
      const recipient = await pxe.getRecipient(completeAddress.address);
      expect(account).toBeUndefined();
      expect(recipient).toEqual(completeAddress);
    });

    it('does not throw when registering the same account twice (just ignores the second attempt)', async () => {
      const keyPair = ConstantKeyPair.random(await Grumpkin.new());
      const completeAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(
        await keyPair.getPrivateKey(),
        Fr.random(),
      );

      await pxe.registerAccount(await keyPair.getPrivateKey(), completeAddress.partialAddress);
      await pxe.registerAccount(await keyPair.getPrivateKey(), completeAddress.partialAddress);
    });

    it('cannot register a recipient with the same aztec address but different pub key or partial address', async () => {
      const recipient1 = await CompleteAddress.random();
      const recipient2 = new CompleteAddress(recipient1.address, Point.random(), Fr.random());

      await pxe.registerRecipient(recipient1);
      await expect(() => pxe.registerRecipient(recipient2)).rejects.toThrow(
        `Complete address with aztec address ${recipient1.address}`,
      );
    });

    it('does not throw when registering the same recipient twice (just ignores the second attempt)', async () => {
      const completeAddress = await CompleteAddress.random();

      await pxe.registerRecipient(completeAddress);
      await pxe.registerRecipient(completeAddress);
    });

    it('successfully adds a contract', async () => {
      const contracts: DeployedContract[] = [await randomDeployedContract(), await randomDeployedContract()];
      await pxe.addContracts(contracts);

      const expectedContractAddresses = contracts.map(contract => contract.completeAddress.address);
      const contractAddresses = await pxe.getContracts();

      // check if all the contracts were returned
      expect(contractAddresses).toEqual(expect.arrayContaining(expectedContractAddresses));
    });

    it('throws when simulating a tx targeting public entrypoint', async () => {
      const functionData = FunctionData.empty();
      functionData.isPrivate = false;
      const txExecutionRequest = TxExecutionRequest.from({
        origin: AztecAddress.random(),
        argsHash: new Fr(0),
        functionData,
        txContext: TxContext.empty(),
        packedArguments: [],
        authWitnesses: [],
      });

      await expect(async () => await pxe.simulateTx(txExecutionRequest, false)).rejects.toThrow(
        'Public entrypoints are not allowed',
      );
    });

    // Note: Not testing a successful run of `simulateTx`, `sendTx`, `getTxReceipt` and `viewTx` here as it requires
    //       a larger setup and it's sufficiently tested in the e2e tests.

    it('throws when getting public storage for non-existent contract', async () => {
      const contract = AztecAddress.random();
      await expect(async () => await pxe.getPublicStorageAt(contract, new Fr(0n))).rejects.toThrow(
        `Contract ${contract.toString()} is not deployed`,
      );
    });

    // Note: Not testing `getExtendedContractData`, `getContractData` and `getUnencryptedLogs` here as these
    //       functions only call AztecNode and these methods are frequently used by the e2e tests.

    it('successfully gets a block number', async () => {
      const blockNum = await pxe.getBlockNumber();
      expect(blockNum).toBeGreaterThanOrEqual(INITIAL_L2_BLOCK_NUM);
    });

    it('successfully gets node info', async () => {
      const nodeInfo = await pxe.getNodeInfo();
      expect(typeof nodeInfo.protocolVersion).toEqual('number');
      expect(typeof nodeInfo.chainId).toEqual('number');
      expect(nodeInfo.l1ContractAddresses.rollupAddress.toString()).toMatch(/0x[a-fA-F0-9]+/);
    });

    // Note: Not testing `isGlobalStateSynchronized`, `isAccountStateSynchronized` and `getSyncStatus` as these methods
    //       only call synchronizer.
  });
};
