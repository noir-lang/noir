import { ReaderContract } from '@aztec/noir-contracts.js';

import { TokenContractTest } from './token_contract_test.js';

const toString = (val: bigint[]) => {
  let str = '';
  for (let i = 0; i < val.length; i++) {
    if (val[i] != 0n) {
      str += String.fromCharCode(Number(val[i]));
    }
  }
  return str;
};

describe('e2e_token_contract reading constants', () => {
  const t = new TokenContractTest('reading_constants');
  const { TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL } = TokenContractTest;
  // Do not destructure anything mutable.
  const { logger } = t;
  let reader: ReaderContract;

  beforeAll(async () => {
    await t.applyBaseSnapshots();

    await t.snapshot(
      'reading_constants',
      async () => {
        logger.verbose('Deploying ReaderContract...');
        const reader = await ReaderContract.deploy(t.wallets[0]).send().deployed();
        logger.verbose(`Deployed ReaderContract to ${reader.address}.`);
        return { readerAddress: reader.address };
      },
      async ({ readerAddress }) => {
        reader = await ReaderContract.at(readerAddress, t.wallets[0]);
        logger.verbose(`Reader contract restored to ${readerAddress}.`);
      },
    );

    await t.setup();
  });

  afterAll(async () => {
    await t.teardown();
  });

  beforeEach(async () => {});

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('check name private', async () => {
    const name = toString(await t.asset.methods.un_get_name().simulate());
    expect(name).toBe(TOKEN_NAME);

    await reader.methods.check_name_private(t.asset.address, TOKEN_NAME).send().wait();
    await expect(reader.methods.check_name_private(t.asset.address, 'WRONG_NAME').simulate()).rejects.toThrow(
      'name.is_eq(_what)',
    );
  });

  it('check name public', async () => {
    const name = toString(await t.asset.methods.un_get_name().simulate());
    expect(name).toBe(TOKEN_NAME);

    await reader.methods.check_name_public(t.asset.address, TOKEN_NAME).send().wait();
    await expect(reader.methods.check_name_public(t.asset.address, 'WRONG_NAME').simulate()).rejects.toThrow(
      'name.is_eq(_what)',
    );
  });

  it('check symbol private', async () => {
    const sym = toString(await t.asset.methods.un_get_symbol().simulate());
    expect(sym).toBe(TOKEN_SYMBOL);

    await reader.methods.check_symbol_private(t.asset.address, TOKEN_SYMBOL).send().wait();

    await expect(reader.methods.check_symbol_private(t.asset.address, 'WRONG_SYMBOL').simulate()).rejects.toThrow(
      "Cannot satisfy constraint 'symbol.is_eq(_what)'",
    );
  });

  it('check symbol public', async () => {
    const sym = toString(await t.asset.methods.un_get_symbol().simulate());
    expect(sym).toBe(TOKEN_SYMBOL);

    await reader.methods.check_symbol_public(t.asset.address, TOKEN_SYMBOL).send().wait();

    await expect(reader.methods.check_symbol_public(t.asset.address, 'WRONG_SYMBOL').simulate()).rejects.toThrow(
      "Failed to solve brillig function 'symbol.is_eq(_what)'",
    );
  });

  it('check decimals private', async () => {
    const dec = await t.asset.methods.un_get_decimals().simulate();
    expect(dec).toBe(TOKEN_DECIMALS);

    await reader.methods.check_decimals_private(t.asset.address, TOKEN_DECIMALS).send().wait();

    await expect(reader.methods.check_decimals_private(t.asset.address, 99).simulate()).rejects.toThrow(
      "Cannot satisfy constraint 'result == what'",
    );
  });

  it('check decimals public', async () => {
    const dec = await t.asset.methods.un_get_decimals().simulate();
    expect(dec).toBe(TOKEN_DECIMALS);

    await reader.methods.check_decimals_public(t.asset.address, TOKEN_DECIMALS).send().wait();

    await expect(reader.methods.check_decimals_public(t.asset.address, 99).simulate()).rejects.toThrow(
      "Failed to solve brillig function 'ret == what'",
    );
  });
});
