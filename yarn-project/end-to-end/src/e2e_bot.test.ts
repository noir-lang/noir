import { Fr, type PXE } from '@aztec/aztec.js';
import { Bot, type BotConfig, getBotDefaultConfig } from '@aztec/bot';

import { setup } from './fixtures/utils.js';

describe('e2e_bot', () => {
  let pxe: PXE;
  let teardown: () => Promise<void>;

  let bot: Bot;
  let config: BotConfig;

  beforeAll(async () => {
    ({ teardown, pxe } = await setup(0));
    const senderPrivateKey = Fr.random();
    config = getBotDefaultConfig({ senderPrivateKey });
    bot = await Bot.create(config, { pxe });
  });

  afterAll(() => teardown());

  it('sends token transfers from the bot', async () => {
    await bot.run();
    const balances = await bot.getBalances();
    expect(balances.recipient.privateBalance).toEqual(1n);
    expect(balances.recipient.publicBalance).toEqual(1n);
  });

  it('reuses the same account and token contract', async () => {
    const { wallet, token, recipient } = bot;
    const bot2 = await Bot.create(config, { pxe });
    expect(bot2.wallet.getAddress().toString()).toEqual(wallet.getAddress().toString());
    expect(bot2.token.address.toString()).toEqual(token.address.toString());
    expect(bot2.recipient.toString()).toEqual(recipient.toString());
  });
});
