import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecRPC } from '@aztec/types';

import { CheatCodes } from './cheat_codes.js';
import { setup } from './fixtures/utils.js';

describe('e2e_cheat_codes', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;

  let cc: CheatCodes;

  beforeAll(async () => {
    ({ aztecNode, aztecRpcServer, cheatCodes: cc } = await setup());
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
  });
});
