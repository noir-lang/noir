import { Fr } from '@aztec/circuits.js';

import { TokenContractTest } from './token_contract_test.js';

const toString = ({ value }: { value: bigint }) => {
  const vals: number[] = Array.from(new Fr(value).toBuffer());

  let str = '';
  for (let i = 0; i < vals.length; i++) {
    if (vals[i] != 0) {
      str += String.fromCharCode(Number(vals[i]));
    }
  }
  return str;
};

describe('e2e_token_contract reading constants', () => {
  const t = new TokenContractTest('reading_constants');
  const { TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL } = TokenContractTest;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
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
    const name = toString(await t.asset.methods.private_get_name().simulate());
    expect(name).toBe(TOKEN_NAME);
  });

  it('check name public', async () => {
    const name = toString(await t.asset.methods.public_get_name().simulate());
    expect(name).toBe(TOKEN_NAME);
  });

  it('check symbol private', async () => {
    const sym = toString(await t.asset.methods.private_get_symbol().simulate());
    expect(sym).toBe(TOKEN_SYMBOL);
  });

  it('check symbol public', async () => {
    const sym = toString(await t.asset.methods.public_get_symbol().simulate());
    expect(sym).toBe(TOKEN_SYMBOL);
  });

  it('check decimals private', async () => {
    const dec = await t.asset.methods.private_get_decimals().simulate();
    expect(dec).toBe(TOKEN_DECIMALS);
  });

  it('check decimals public', async () => {
    const dec = await t.asset.methods.public_get_decimals().simulate();
    expect(dec).toBe(TOKEN_DECIMALS);
  });
});
