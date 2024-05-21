import { ImportTestContract, TestContract } from '@aztec/noir-contracts.js';

import { NestedContractTest } from './nested_contract_test.js';

describe('e2e_nested_contract manual', () => {
  const t = new NestedContractTest('manual');
  let testContract: TestContract;
  let importerContract: ImportTestContract;
  let { wallets, logger } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    ({ wallets, logger } = t);
  });

  beforeEach(async () => {
    importerContract = await ImportTestContract.deploy(wallets[0]).send().deployed();
    testContract = await TestContract.deploy(wallets[0]).send().deployed();
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('calls a method with multiple arguments', async () => {
    logger.info(`Calling main on importer contract`);
    await importerContract.methods.main_contract(testContract.address).send().wait();
  });

  it('calls a method no arguments', async () => {
    logger.info(`Calling noargs on importer contract`);
    await importerContract.methods.call_no_args(testContract.address).send().wait();
  });

  it('calls a public function', async () => {
    logger.info(`Calling public_fn on importer contract`);
    await importerContract.methods.call_public_fn(testContract.address).send().wait();
  });

  it('calls a public function from a public function', async () => {
    logger.info(`Calling pub_public_fn on importer contract`);
    await importerContract.methods.pub_call_public_fn(testContract.address).send().wait();
  });
});
