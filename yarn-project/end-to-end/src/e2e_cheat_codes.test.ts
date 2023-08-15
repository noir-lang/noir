import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, EthAddress, Fr } from '@aztec/aztec-rpc';
import { Wallet } from '@aztec/aztec.js';
import { RollupAbi } from '@aztec/l1-artifacts';
import { LendingContract } from '@aztec/noir-contracts/types';
import { AztecRPC } from '@aztec/types';

import { Account, Chain, HttpTransport, PublicClient, WalletClient, getAddress, getContract, parseEther } from 'viem';

import { CheatCodes } from './fixtures/cheat_codes.js';
import { setup } from './fixtures/utils.js';

describe('e2e_cheat_codes', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let cc: CheatCodes;

  let walletClient: WalletClient<HttpTransport, Chain, Account>;
  let publicClient: PublicClient<HttpTransport, Chain>;
  let rollupAddress: EthAddress;
  let recipient: AztecAddress;

  beforeAll(async () => {
    let deployL1ContractsValues;
    let accounts;
    ({ aztecNode, aztecRpcServer, wallet, accounts, cheatCodes: cc, deployL1ContractsValues } = await setup());

    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;
    rollupAddress = deployL1ContractsValues.rollupAddress;
    recipient = accounts[0].address;
  }, 100_000);

  afterAll(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  describe('L1 only', () => {
    describe('mine', () => {
      it(`mine block`, async () => {
        const blockNumber = await cc.l1.blockNumber();
        await cc.l1.mine();
        expect(await cc.l1.blockNumber()).toBe(blockNumber + 1);
      });

      it.each([10, 42, 99])(`mine blocks`, async increment => {
        const blockNumber = await cc.l1.blockNumber();
        await cc.l1.mine(increment);
        expect(await cc.l1.blockNumber()).toBe(blockNumber + increment);
      });
    });

    it.each([100, 42, 99])('setNextBlockTimestamp', async increment => {
      const blockNumber = await cc.l1.blockNumber();
      const timestamp = await cc.l1.timestamp();
      await cc.l1.setNextBlockTimestamp(timestamp + increment);

      expect(await cc.l1.timestamp()).toBe(timestamp);

      await cc.l1.mine();

      expect(await cc.l1.blockNumber()).toBe(blockNumber + 1);
      expect(await cc.l1.timestamp()).toBe(timestamp + increment);
    });

    it('load a value at a particular storage slot', async () => {
      // check that storage slot 0 is empty as expected
      const res = await cc.l1.load(EthAddress.ZERO, 0n);
      expect(res).toBe(0n);
    });

    it.each(['1', 'bc40fbf4394cd00f78fae9763b0c2c71b21ea442c42fdadc5b720537240ebac1'])(
      'store a value at a given slot and its keccak value of the slot (if it were in a map) ',
      async storageSlotInHex => {
        const storageSlot = BigInt('0x' + storageSlotInHex);
        const valueToSet = 5n;
        const contractAddress = EthAddress.fromString('0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266');
        await cc.l1.store(contractAddress, storageSlot, valueToSet);
        expect(await cc.l1.load(contractAddress, storageSlot)).toBe(valueToSet);
        // also test with the keccak value of the slot - can be used to compute storage slots of maps
        await cc.l1.store(contractAddress, cc.l1.keccak256(0n, storageSlot), valueToSet);
        expect(await cc.l1.load(contractAddress, cc.l1.keccak256(0n, storageSlot))).toBe(valueToSet);
      },
    );

    it('set bytecode correctly', async () => {
      const contractAddress = EthAddress.fromString('0x70997970C51812dc3A010C7d01b50e0d17dc79C8');
      await cc.l1.etch(contractAddress, '0x1234');
      expect(await cc.l1.getBytecode(contractAddress)).toBe('0x1234');
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
      await cc.l1.startPrank(EthAddress.fromString(randomAddress));
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
      await cc.l1.stopPrank(EthAddress.fromString(randomAddress));

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

    it('can modify L1 block time', async () => {
      // deploy lending contract
      const tx = LendingContract.deploy(aztecRpcServer).send();
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      const contract = await LendingContract.at(receipt.contractAddress!, wallet);

      // now update time:
      const timestamp = await cc.l1.timestamp();
      const newTimestamp = timestamp + 100_000_000;
      await cc.l2.warp(newTimestamp);

      // ensure rollup contract is correctly updated
      const rollup = getContract({ address: getAddress(rollupAddress.toString()), abi: RollupAbi, publicClient });
      expect(Number(await rollup.read.lastBlockTs())).toEqual(newTimestamp);

      // initialize contract -> this updates `lastUpdatedTs` to the current timestamp of the rollup
      // this should be the new timestamp
      const txInit = contract.methods.init().send({ origin: recipient });
      await txInit.isMined({ interval: 0.1 });

      // fetch last updated ts from L2 contract and expect it to me same as new timestamp
      const lastUpdatedTs = Number(new Fr((await contract.methods.getTot(0).view())[0][1]).value);
      expect(lastUpdatedTs).toEqual(newTimestamp);
      // ensure anvil is correctly updated
      expect(await cc.l1.timestamp()).toEqual(newTimestamp);
      // ensure rollup contract is correctly updated
      expect(Number(await rollup.read.lastBlockTs())).toEqual(newTimestamp);
      expect;
    }, 50_000);
  });
});
