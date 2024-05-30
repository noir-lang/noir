import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract access control', () => {
  const t = new TokenContractTest('access_control');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('Set admin', async () => {
    await t.asset.methods.set_admin(t.accounts[1].address).send().wait();
    expect(await t.asset.methods.admin().simulate()).toBe(t.accounts[1].address.toBigInt());
  });

  it('Add minter as admin', async () => {
    await t.asset.withWallet(t.wallets[1]).methods.set_minter(t.accounts[1].address, true).send().wait();
    expect(await t.asset.methods.is_minter(t.accounts[1].address).simulate()).toBe(true);
  });

  it('Revoke minter as admin', async () => {
    await t.asset.withWallet(t.wallets[1]).methods.set_minter(t.accounts[1].address, false).send().wait();
    expect(await t.asset.methods.is_minter(t.accounts[1].address).simulate()).toBe(false);
  });

  describe('failure cases', () => {
    it('Set admin (not admin)', async () => {
      await expect(t.asset.methods.set_admin(t.accounts[0].address).simulate()).rejects.toThrow(
        'Assertion failed: caller is not admin',
      );
    });
    it('Revoke minter not as admin', async () => {
      await expect(t.asset.methods.set_minter(t.accounts[0].address, false).simulate()).rejects.toThrow(
        'Assertion failed: caller is not admin',
      );
    });
  });
});
