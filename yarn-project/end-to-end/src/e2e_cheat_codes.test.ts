import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, EthAddress } from '@aztec/aztec-rpc';
import { AztecRPC } from '@aztec/types';

import { Account, Chain, HttpTransport, PublicClient, WalletClient, parseEther } from 'viem';

import { CheatCodes } from './fixtures/cheat_codes.js';
import { setup } from './fixtures/utils.js';

describe('e2e_cheat_codes', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;

  let cc: CheatCodes;
  let walletClient: WalletClient<HttpTransport, Chain, Account>;
  let publicClient: PublicClient<HttpTransport, Chain>;

  beforeAll(async () => {
    let deployL1ContractsValues;
    ({ aztecNode, aztecRpcServer, cheatCodes: cc, deployL1ContractsValues } = await setup());
    walletClient = deployL1ContractsValues.walletClient;
    publicClient = deployL1ContractsValues.publicClient;
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
      const randomAddress = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2';
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
      await walletClient.sendTransaction({
        account: randomAddress,
        to: myAddress,
        value: amountToSend,
      });
      expect(await publicClient.getBalance({ address: randomAddress })).toBeLessThan(beforeBalance - amountToSend); // account for fees too

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
  });
});
