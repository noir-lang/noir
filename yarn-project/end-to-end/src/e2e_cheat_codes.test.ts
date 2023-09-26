import { EthAddress } from '@aztec/aztec-rpc';
import { CheatCodes, Wallet } from '@aztec/aztec.js';
import { RollupAbi } from '@aztec/l1-artifacts';
import { TestContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { Account, Chain, HttpTransport, PublicClient, WalletClient, getAddress, getContract, parseEther } from 'viem';

import { setup } from './fixtures/utils.js';

describe('e2e_cheat_codes', () => {
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let cc: CheatCodes;
  let teardown: () => Promise<void>;

  let walletClient: WalletClient<HttpTransport, Chain, Account>;
  let publicClient: PublicClient<HttpTransport, Chain>;
  let rollupAddress: EthAddress;

  beforeAll(async () => {
    let deployL1ContractsValues;
    ({ teardown, aztecRpcServer, wallet, cheatCodes: cc, deployL1ContractsValues } = await setup());

    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;
    rollupAddress = deployL1ContractsValues.l1ContractAddresses.rollupAddress;
  }, 100_000);

  afterAll(() => teardown());

  describe('L1 only', () => {
    describe('mine', () => {
      it(`mine block`, async () => {
        const blockNumber = await cc.eth.blockNumber();
        await cc.eth.mine();
        expect(await cc.eth.blockNumber()).toBe(blockNumber + 1);
      });

      it.each([10, 42, 99])(`mine blocks`, async increment => {
        const blockNumber = await cc.eth.blockNumber();
        await cc.eth.mine(increment);
        expect(await cc.eth.blockNumber()).toBe(blockNumber + increment);
      });
    });

    it.each([100, 42, 99])('setNextBlockTimestamp', async increment => {
      const blockNumber = await cc.eth.blockNumber();
      const timestamp = await cc.eth.timestamp();
      await cc.eth.setNextBlockTimestamp(timestamp + increment);

      expect(await cc.eth.timestamp()).toBe(timestamp);

      await cc.eth.mine();

      expect(await cc.eth.blockNumber()).toBe(blockNumber + 1);
      expect(await cc.eth.timestamp()).toBe(timestamp + increment);
    });

    it('setNextBlockTimestamp to a past timestamp throws', async () => {
      const timestamp = await cc.eth.timestamp();
      const pastTimestamp = timestamp - 1000;
      await expect(async () => await cc.eth.setNextBlockTimestamp(pastTimestamp)).rejects.toThrow(
        `Error setting next block timestamp: Timestamp error: ${pastTimestamp} is lower than or equal to previous block's timestamp`,
      );
    });

    it('load a value at a particular storage slot', async () => {
      // check that storage slot 0 is empty as expected
      const res = await cc.eth.load(EthAddress.ZERO, 0n);
      expect(res).toBe(0n);
    });

    it.each(['1', 'bc40fbf4394cd00f78fae9763b0c2c71b21ea442c42fdadc5b720537240ebac1'])(
      'store a value at a given slot and its keccak value of the slot (if it were in a map) ',
      async storageSlotInHex => {
        const storageSlot = BigInt('0x' + storageSlotInHex);
        const valueToSet = 5n;
        const contractAddress = EthAddress.fromString('0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266');
        await cc.eth.store(contractAddress, storageSlot, valueToSet);
        expect(await cc.eth.load(contractAddress, storageSlot)).toBe(valueToSet);
        // also test with the keccak value of the slot - can be used to compute storage slots of maps
        await cc.eth.store(contractAddress, cc.eth.keccak256(0n, storageSlot), valueToSet);
        expect(await cc.eth.load(contractAddress, cc.eth.keccak256(0n, storageSlot))).toBe(valueToSet);
      },
    );

    it('set bytecode correctly', async () => {
      const contractAddress = EthAddress.fromString('0x70997970C51812dc3A010C7d01b50e0d17dc79C8');
      await cc.eth.etch(contractAddress, '0x1234');
      expect(await cc.eth.getBytecode(contractAddress)).toBe('0x1234');
    });

    it('impersonate', async () => {
      // we will transfer 1 eth to a random address. Then impersonate the address to be able to send funds
      // without impersonation we wouldn't be able to send funds.
      const myAddress = (await walletClient.getAddresses())[0];
      const randomAddress = EthAddress.random().toString();
      await walletClient.sendTransaction({
        account: myAddress,
        to: randomAddress,
        value: parseEther('1'),
      });
      const beforeBalance = await publicClient.getBalance({ address: randomAddress });

      // impersonate random address
      await cc.eth.startImpersonating(EthAddress.fromString(randomAddress));
      // send funds from random address
      const amountToSend = parseEther('0.1');
      const txHash = await walletClient.sendTransaction({
        account: randomAddress,
        to: myAddress,
        value: amountToSend,
      });
      const tx = await publicClient.waitForTransactionReceipt({ hash: txHash });
      const feePaid = tx.gasUsed * tx.effectiveGasPrice;
      expect(await publicClient.getBalance({ address: randomAddress })).toBe(beforeBalance - amountToSend - feePaid);

      // stop impersonating
      await cc.eth.stopImpersonating(EthAddress.fromString(randomAddress));

      // making calls from random address should not be successful
      try {
        await walletClient.sendTransaction({
          account: randomAddress,
          to: myAddress,
          value: amountToSend,
        });
        // done with a try-catch because viem errors are noisy and we need to check just a small portion of the error.
        fail('should not be able to send funds from random address');
      } catch (e: any) {
        expect(e.message).toContain('No Signer available');
      }
    });

    it('can modify L2 block time', async () => {
      const tx = TestContract.deploy(aztecRpcServer).send();
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      const contract = await TestContract.at(receipt.contractAddress!, wallet);

      // now update time:
      const timestamp = await cc.eth.timestamp();
      const newTimestamp = timestamp + 100_000_000;
      await cc.aztec.warp(newTimestamp);

      // ensure rollup contract is correctly updated
      const rollup = getContract({ address: getAddress(rollupAddress.toString()), abi: RollupAbi, publicClient });
      expect(Number(await rollup.read.lastBlockTs())).toEqual(newTimestamp);
      expect(Number(await rollup.read.lastWarpedBlockTs())).toEqual(newTimestamp);

      const txIsTimeEqual = contract.methods.isTimeEqual(newTimestamp).send();
      const isTimeEqualReceipt = await txIsTimeEqual.wait({ interval: 0.1 });
      expect(isTimeEqualReceipt.status).toBe(TxStatus.MINED);

      // Since last rollup block was warped, txs for this rollup will have time incremented by 1
      // See https://github.com/AztecProtocol/aztec-packages/issues/1614 for details
      const txTimeNotEqual = contract.methods.isTimeEqual(newTimestamp + 1).send();
      const isTimeNotEqualReceipt = await txTimeNotEqual.wait({ interval: 0.1 });
      expect(isTimeNotEqualReceipt.status).toBe(TxStatus.MINED);
      // block is published at t >= newTimestamp + 1.
      expect(Number(await rollup.read.lastBlockTs())).toBeGreaterThanOrEqual(newTimestamp + 1);
    }, 50_000);

    it('should throw if setting L2 block time to a past timestamp', async () => {
      const timestamp = await cc.eth.timestamp();
      const pastTimestamp = timestamp - 1000;
      await expect(async () => await cc.aztec.warp(pastTimestamp)).rejects.toThrow(
        `Error setting next block timestamp: Timestamp error: ${pastTimestamp} is lower than or equal to previous block's timestamp`,
      );
    });
  });
});
