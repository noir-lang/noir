import { AztecAddress, Fr } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { openTmpStore } from '@aztec/kv-store/utils';

import { NewTestKeyStore } from './new_test_key_store.js';

describe('NewTestKeyStore', () => {
  it('Adds account and returns keys', async () => {
    const db = openTmpStore();
    const keyStore = new NewTestKeyStore(new Grumpkin(), db);

    // Arbitrary fixed values
    const sk = new Fr(8923n);
    const partialAddress = new Fr(243523n);

    const accountAddress = await keyStore.addAccount(sk, partialAddress);
    expect(accountAddress.toString()).toMatchInlineSnapshot(
      `"0x0ba7834252d19c4f09d29303c269f303f40ae3d2043f921ed0bf8c0709926d4e"`,
    );

    const masterNullifierPublicKey = await keyStore.getMasterNullifierPublicKey(accountAddress);
    expect(masterNullifierPublicKey.toString()).toMatchInlineSnapshot(
      `"0x2ef5d15dd65d29546680ab72846fb071f41cb9f2a0212215e6c560e29df4ff650ce764818364b376be92dc2f49577fe440e64a16012584f7c4ee94f7edbc323a"`,
    );

    const masterIncomingViewingPublicKey = await keyStore.getMasterIncomingViewingPublicKey(accountAddress);
    expect(masterIncomingViewingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x1c088f4e4a711f236a88b55da9ddf388de0bc00d56a5ceca96cea3a5cbe75bf32db0a333ba30c36b844d9fc6d2fb0de8d10e4371f0c5baebae452d90ff366798"`,
    );

    const masterOutgoingViewingPublicKey = await keyStore.getMasterOutgoingViewingPublicKey(accountAddress);
    expect(masterOutgoingViewingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x232d0b445d097fbc2046012c3fc474f6a9beef97eda1d8d1f2487dbe501ee1e70e8db9a824531a14e8717dee54cbb7abfec29a88c550a49617258bd6fd858242"`,
    );

    const masterTaggingPublicKey = await keyStore.getMasterTaggingPublicKey(accountAddress);
    expect(masterTaggingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x076429010fdebfa522b053267f654a4c5daf18589915d96f7e5001d63ea2033f27f915f254560c84450aa38e93c3162be52492d05b316e75f542e3b302117360"`,
    );

    // Arbitrary app contract address
    const appAddress = AztecAddress.fromBigInt(624n);

    const appNullifierSecretKey = await keyStore.getAppNullifierSecretKey(accountAddress, appAddress);
    expect(appNullifierSecretKey.toString()).toMatchInlineSnapshot(
      `"0x230a44dfe7cfec7a735c89f7289c5cb5d2c3dc0bf5d3505917fd2476f67873a8"`,
    );

    const appIncomingViewingSecretKey = await keyStore.getAppIncomingViewingSecretKey(accountAddress, appAddress);
    expect(appIncomingViewingSecretKey.toString()).toMatchInlineSnapshot(
      `"0x0084c92262407236c992dcea10cf3406a642074cad6c6034d2990ffb073207a7"`,
    );

    const appOutgoingViewingSecretKey = await keyStore.getAppOutgoingViewingSecretKey(accountAddress, appAddress);
    expect(appOutgoingViewingSecretKey.toString()).toMatchInlineSnapshot(
      `"0x2639b26510f9d30b7e173d301b263b246b7a576186be1f44cd7c86bc06773f8a"`,
    );
  });
});
