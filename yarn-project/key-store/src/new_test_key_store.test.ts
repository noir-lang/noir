import { Fr } from '@aztec/circuits.js';
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
      `"0x2e34847ad9019320ac89a6ec9b42fec90f94ef4162fdfdd7f5b7668e32d82655"`,
    );

    // TODO(#5714): The keys are currently the same here because separator is currently ignored in poseidon
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
  });
});
