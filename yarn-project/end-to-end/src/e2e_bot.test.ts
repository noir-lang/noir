import { Fr, type PXE } from '@aztec/aztec.js';
import { Bot, type BotConfig } from '@aztec/bot';

import { getBotDefaultConfig } from '../../bot/src/config.js';
import { setup } from './fixtures/utils.js';

describe('e2e_bot', () => {
  let pxe: PXE;
  let teardown: () => Promise<void>;

  let bot: Bot;
  let config: BotConfig;

  beforeAll(async () => {
    ({ teardown, pxe } = await setup(0));
    const senderPrivateKey = Fr.random();
    config = {
      ...getBotDefaultConfig(),
      ...senderPrivateKey,
    };
    bot = await Bot.create(config, { pxe });
  });

  afterAll(() => teardown());

  it('sends token transfers from the bot', async () => {
    const { recipient: recipientBefore } = await bot.getBalances();

    await bot.run();
    const { recipient: recipientAfter } = await bot.getBalances();
    expect(recipientAfter.privateBalance - recipientBefore.privateBalance).toEqual(1n);
    expect(recipientAfter.publicBalance - recipientBefore.publicBalance).toEqual(1n);
  });

  it('sends token transfers with hardcoded gas and no simulation', async () => {
    bot.updateConfig({ daGasLimit: 1e9, l2GasLimit: 1e9, skipPublicSimulation: true });
    const { recipient: recipientBefore } = await bot.getBalances();

    await bot.run();
    const { recipient: recipientAfter } = await bot.getBalances();
    expect(recipientAfter.privateBalance - recipientBefore.privateBalance).toEqual(1n);
    expect(recipientAfter.publicBalance - recipientBefore.publicBalance).toEqual(1n);
  });

  it('reuses the same account and token contract', async () => {
    const { wallet, token, recipient } = bot;
    const bot2 = await Bot.create(config, { pxe });
    expect(bot2.wallet.getAddress().toString()).toEqual(wallet.getAddress().toString());
    expect(bot2.token.address.toString()).toEqual(token.address.toString());
    expect(bot2.recipient.toString()).toEqual(recipient.toString());
  });
});
