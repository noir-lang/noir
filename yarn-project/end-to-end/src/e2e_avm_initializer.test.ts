import { type Wallet } from '@aztec/aztec.js';
import { AvmInitializerTestContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('e2e_avm_initializer', () => {
  jest.setTimeout(TIMEOUT);

  let wallet: Wallet;
  let avmContact: AvmInitializerTestContract;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
  }, 100_000);

  afterAll(() => teardown());

  beforeEach(async () => {
    avmContact = await AvmInitializerTestContract.deploy(wallet).send().deployed();
  }, 50_000);

  describe('Storage', () => {
    it('Read immutable (initialized) storage (Field)', async () => {
      expect(await avmContact.methods.view_storage_immutable().simulate()).toEqual(42n);
    });
  });
});
