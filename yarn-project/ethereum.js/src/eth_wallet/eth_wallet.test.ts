import { EthAddress } from '@aztec/foundation/eth-address';
import { EthAccount } from '../eth_account/index.js';
import { EthWallet } from './eth_wallet.js';

describe('wallet', () => {
  const address = EthAddress.fromString('0xEB014f8c8B418Db6b45774c326A0E64C78914dC0');
  const privateKey = Buffer.from('be6383dad004f233317e46ddb46ad31b16064d14447a95cc1d8c8d4bc61c3728', 'hex');

  it('creates the right number of wallets', () => {
    const wallet = new EthWallet();
    expect(wallet.length).toBe(0);

    wallet.create(2, Buffer.from('542342f!@#$$'));
    expect(wallet.length).toBe(2);

    wallet.create(3);
    expect(wallet.length).toBe(5);

    expect(EthAddress.isAddress(wallet.accounts[1].address.toString())).toBe(true);
    expect(EthAddress.isAddress(wallet.accounts[2].address.toString())).toBe(true);
  });

  it('add wallet using a privatekey', () => {
    const wallet = new EthWallet();

    const account = wallet.add(privateKey);

    expect(account.address).toEqual(address);
    expect(account.privateKey).toEqual(privateKey);
    expect(wallet.getAccountIndex(account.address)).toBe(0);

    expect(wallet.getAccount(address)!.address).toEqual(address);
    expect(wallet.getAccount(0)!.address).toEqual(address);
    expect(wallet.length).toBe(1);
  });

  it('add wallet using an account', () => {
    const wallet = new EthWallet();

    const account = new EthAccount(privateKey);
    wallet.add(account);

    expect(account.address).toEqual(address);
    expect(account.privateKey).toEqual(privateKey);
    expect(wallet.getAccountIndex(account.address)).toBe(0);

    expect(wallet.getAccount(address)!.address).toEqual(address);
    expect(wallet.getAccount(0)!.address).toEqual(address);
    expect(wallet.length).toBe(1);
  });

  it('should not add wallet twice work', () => {
    const wallet = new EthWallet();

    const account = new EthAccount(privateKey);
    wallet.add(account);
    wallet.add(account);

    expect(account.address).toEqual(address);
    expect(account.privateKey).toEqual(privateKey);
    expect(wallet.getAccountIndex(account.address)).toBe(0);

    expect(wallet.getAccount(address)!.address).toEqual(address);
    expect(wallet.getAccount(0)!.address).toEqual(address);
    expect(wallet.length).toBe(1);
  });

  it('remove wallet using an index', () => {
    const wallet = new EthWallet();

    wallet.add(privateKey);
    expect(wallet.length).toBe(1);

    wallet.remove(0);
    expect(wallet.getAccount(address)).toBeUndefined();
    expect(wallet.getAccount(0)).toBeUndefined();
    expect(wallet.length).toBe(0);
  });

  it('remove wallet using an address', () => {
    const wallet = new EthWallet();

    wallet.add(privateKey);
    expect(wallet.length).toBe(1);

    wallet.remove(address);
    expect(wallet.length).toBe(0);
  });

  it('create 5 wallets, remove two, create two more and check for overwrites', () => {
    const count = 5;
    const wallet = new EthWallet();
    expect(wallet.length).toBe(0);

    wallet.create(count);
    const initialAddresses = [0, 1, 2, 3, 4].map(n => wallet.getAccount(n)!.address);
    expect(wallet.length).toBe(count);

    const remainingAddresses = [0, 1, 3];
    const beforeRemoval = remainingAddresses.map(n => wallet.getAccount(n)!.address);

    wallet.remove(2);
    wallet.remove(4);

    expect(wallet.getAccount(2)).toBeUndefined();
    expect(wallet.getAccount(4)).toBeUndefined();

    const afterRemoval = remainingAddresses.map(n => wallet.getAccount(n)!.address);

    expect(wallet.length).toBe(3);

    wallet.create(2);
    expect(EthAddress.isAddress(wallet.accounts[2].address.toString())).toBe(true);
    expect(EthAddress.isAddress(wallet.accounts[4].address.toString())).toBe(true);
    expect(wallet.getAccount(5)).toBeUndefined();

    const afterMoreCreation = remainingAddresses.map(n => wallet.getAccount(n)!.address);
    const newAddresses = [0, 1, 2, 3, 4].map(n => wallet.getAccount(n)!.address);

    // Checks for account overwrites
    expect(wallet.length).toBe(count);
    expect(beforeRemoval).toEqual(afterMoreCreation);
    expect(afterRemoval).toEqual(afterMoreCreation);
    expect(initialAddresses).not.toEqual(newAddresses);
  });

  it('clear wallet', () => {
    const count = 10;
    const wallet = new EthWallet();

    wallet.create(count);
    expect(wallet.length).toBe(10);

    wallet.clear();

    for (let i = 0; i < count; i++) {
      expect(wallet.getAccount(i)).toBeUndefined();
    }
    expect(wallet.length).toBe(0);
  });

  it('encrypt then decrypt wallet', async () => {
    const wallet = new EthWallet();
    const password = 'qwerty';

    wallet.create(5);
    const addressFromWallet = wallet.accounts[0].address;
    expect(wallet.length).toBe(5);

    wallet.remove(2);
    expect(wallet.length).toBe(4);

    const keystore = await wallet.encrypt(password);
    expect(wallet.length).toBe(4);

    wallet.clear();
    expect(wallet.length).toBe(0);

    await wallet.decrypt(keystore, password);
    expect(wallet.length).toBe(4);

    const addressFromKeystore = wallet.accounts[0].address;
    expect(addressFromKeystore).toEqual(addressFromWallet);
  }, 30000);

  it('should create correct accounts from mnemonic', () => {
    const mnemonic = 'profit gather crucial census birth effort clinic roast harvest rebuild hidden bamboo';
    const addresses = [
      '0xa97ab6ec66bc2354a7d880bae18fea633752ca85',
      '0x7048779748e8899c8f8baa9dd6c8973411d0fa17',
      '0xe8d62adfc3584a444546f17cd1bb3c327767edb0',
      '0x951afb198aaa10702f456bcc61aa8f59c4f17a2f',
      '0x0598ce5f520574b5b8bd9651971c7767e4354189',
      '0xa2e8c16c765ab30900e205a7ea240df7cbe63548',
      '0x107d4df66df086faaa66690fadd5d3ed1ca630d1',
      '0x070b4ed7bee40216355cf84d88a7ab2696caf373',
      '0x87fa6ff918e36b7b73ed99c1ae5e7c3d63edb44b',
      '0xc174aec38d282396604130e65b59d0096ca53fd7',
    ];

    const wallet = EthWallet.fromMnemonic(mnemonic, 10);

    expect(wallet.accounts.map(a => a.address.toString().toLowerCase())).toEqual(addresses);

    addresses.forEach((address, i) => {
      expect(wallet.getAccount(i)!.address.toString().toLowerCase()).toBe(address);
    });
  });
});
