import {
  AztecAddress,
  Fq,
  Fr,
  computeAppNullifierSecretKey,
  deriveKeys,
  derivePublicKeyFromSecretKey,
} from '@aztec/circuits.js';
import { openTmpStore } from '@aztec/kv-store/utils';

import { KeyStore } from './key_store.js';

describe('KeyStore', () => {
  it('Adds account and returns keys', async () => {
    const keyStore = new KeyStore(openTmpStore());

    // Arbitrary fixed values
    const sk = new Fr(8923n);
    const keys = deriveKeys(sk);
    const derivedMasterNullifierPublicKey = derivePublicKeyFromSecretKey(keys.masterNullifierSecretKey);
    const computedMasterNullifierPublicKeyHash = derivedMasterNullifierPublicKey.hash();

    const partialAddress = new Fr(243523n);

    const { address: accountAddress } = await keyStore.addAccount(sk, partialAddress);
    expect(accountAddress.toString()).toMatchInlineSnapshot(
      `"0x23a1f3daede8d7ff24ccd0354b114c38c11a1512434a2529cf404f74a8f0ab6c"`,
    );

    const { pkM: masterNullifierPublicKey } = await keyStore.getKeyValidationRequest(
      computedMasterNullifierPublicKeyHash,
      AztecAddress.random(), // Address is random because we are not interested in the app secret key here
    );
    expect(masterNullifierPublicKey.toString()).toMatchInlineSnapshot(
      `"0x1c088f4e4a711f236a88b55da9ddf388de0bc00d56a5ceca96cea3a5cbe75bf32db0a333ba30c36b844d9fc6d2fb0de8d10e4371f0c5baebae452d90ff366798"`,
    );

    const masterIncomingViewingPublicKey = await keyStore.getMasterIncomingViewingPublicKey(accountAddress);
    expect(masterIncomingViewingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x232d0b445d097fbc2046012c3fc474f6a9beef97eda1d8d1f2487dbe501ee1e70e8db9a824531a14e8717dee54cbb7abfec29a88c550a49617258bd6fd858242"`,
    );

    const masterOutgoingViewingPublicKey = await keyStore.getMasterOutgoingViewingPublicKey(accountAddress);
    expect(masterOutgoingViewingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x076429010fdebfa522b053267f654a4c5daf18589915d96f7e5001d63ea2033f27f915f254560c84450aa38e93c3162be52492d05b316e75f542e3b302117360"`,
    );

    const masterTaggingPublicKey = await keyStore.getMasterTaggingPublicKey(accountAddress);
    expect(masterTaggingPublicKey.toString()).toMatchInlineSnapshot(
      `"0x07cec19d32f1cbaaacf16edc081021b696c86dff14160779373ffc77b04568e7076f25b0e7f0d02fd6433d788483e2262c1e45c5962790b40d1cd7efbd5253d3"`,
    );

    // Arbitrary app contract address
    const appAddress = AztecAddress.fromBigInt(624n);

    const { pkM: obtainedMasterNullifierPublicKey, skApp: appNullifierSecretKey } =
      await keyStore.getKeyValidationRequest(computedMasterNullifierPublicKeyHash, appAddress);
    expect(appNullifierSecretKey.toString()).toMatchInlineSnapshot(
      `"0x0084c92262407236c992dcea10cf3406a642074cad6c6034d2990ffb073207a7"`,
    );
    expect(obtainedMasterNullifierPublicKey).toEqual(masterNullifierPublicKey);

    const appIncomingViewingSecretKey = await keyStore.getAppIncomingViewingSecretKey(accountAddress, appAddress);
    expect(appIncomingViewingSecretKey.toString()).toMatchInlineSnapshot(
      `"0x2639b26510f9d30b7e173d301b263b246b7a576186be1f44cd7c86bc06773f8a"`,
    );

    const appOutgoingViewingSecretKey = await keyStore.getAppOutgoingViewingSecretKey(accountAddress, appAddress);
    expect(appOutgoingViewingSecretKey.toString()).toMatchInlineSnapshot(
      `"0x13b400d2fccab28a04a4df9fe541d242e6b518d03137ef0ffa57c3d98cc56e67"`,
    );

    // Returned accounts are as expected
    const accounts = await keyStore.getAccounts();
    expect(accounts.toString()).toMatchInlineSnapshot(
      `"0x23a1f3daede8d7ff24ccd0354b114c38c11a1512434a2529cf404f74a8f0ab6c"`,
    );

    // Manages to find master nullifer secret key for pub key
    const masterNullifierSecretKey = await keyStore.getMasterSecretKey(masterNullifierPublicKey);
    expect(masterNullifierSecretKey.toString()).toMatchInlineSnapshot(
      `"0x1f1f43082427fed511393bbabf8a471eb87af09f0e95bb740dc33e1ced1a54c1"`,
    );

    // Manages to find master incoming viewing secret key for pub key
    const masterIncomingViewingSecretKey = await keyStore.getMasterSecretKey(masterIncomingViewingPublicKey);
    expect(masterIncomingViewingSecretKey.toString()).toMatchInlineSnapshot(
      `"0x1d1d920024dd64e019c23de36d27aefe4d9d4d05983b99cf85bea9e85fd60020"`,
    );
  });

  it('nullifier key rotation tests', async () => {
    const keyStore = new KeyStore(openTmpStore());

    // Arbitrary fixed values
    const sk = new Fr(8923n);
    const partialAddress = new Fr(243523n);

    const { address: accountAddress } = await keyStore.addAccount(sk, partialAddress);
    expect(accountAddress.toString()).toMatchInlineSnapshot(
      `"0x23a1f3daede8d7ff24ccd0354b114c38c11a1512434a2529cf404f74a8f0ab6c"`,
    );

    // Arbitrary fixed values
    const newMasterNullifierSecretKeys = [new Fq(420n), new Fq(69n), new Fq(42069n)];
    const newDerivedMasterNullifierPublicKeys = [
      derivePublicKeyFromSecretKey(newMasterNullifierSecretKeys[0]),
      derivePublicKeyFromSecretKey(newMasterNullifierSecretKeys[1]),
      derivePublicKeyFromSecretKey(newMasterNullifierSecretKeys[2]),
    ];

    const newComputedMasterNullifierPublicKeyHashes = [
      newDerivedMasterNullifierPublicKeys[0].hash(),
      newDerivedMasterNullifierPublicKeys[1].hash(),
      newDerivedMasterNullifierPublicKeys[2].hash(),
    ];

    // We rotate our nullifier key
    await keyStore.rotateMasterNullifierKey(accountAddress, newMasterNullifierSecretKeys[0]);
    await keyStore.rotateMasterNullifierKey(accountAddress, newMasterNullifierSecretKeys[1]);
    await keyStore.rotateMasterNullifierKey(accountAddress, newMasterNullifierSecretKeys[2]);

    // We make sure we can get master nullifier public keys with master nullifier public key hashes
    const { pkM: masterNullifierPublicKey2 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[2],
      AztecAddress.random(), // Address is random because we are not interested in the app secret key here
    );
    expect(masterNullifierPublicKey2).toEqual(newDerivedMasterNullifierPublicKeys[2]);
    const { pkM: masterNullifierPublicKey1 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[1],
      AztecAddress.random(), // Address is random because we are not interested in the app secret key here
    );
    expect(masterNullifierPublicKey1).toEqual(newDerivedMasterNullifierPublicKeys[1]);
    const { pkM: masterNullifierPublicKey0 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[0],
      AztecAddress.random(), // Address is random because we are not interested in the app secret key here
    );
    expect(masterNullifierPublicKey0).toEqual(newDerivedMasterNullifierPublicKeys[0]);

    // Arbitrary app contract address
    const appAddress = AztecAddress.fromBigInt(624n);

    // We make sure we can get app nullifier secret keys with master nullifier public key hashes
    const { skApp: appNullifierSecretKey0 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[0],
      appAddress,
    );
    expect(appNullifierSecretKey0.toString()).toMatchInlineSnapshot(
      `"0x21e3ca4bc7ae2b5e9fe343f4eec5c0aa7391857333821a4b0a1c7d4cb0055bf0"`,
    );
    const { skApp: appNullifierSecretKey1 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[1],
      appAddress,
    );
    expect(appNullifierSecretKey1.toString()).toMatchInlineSnapshot(
      `"0x0900aea4825d057e5bc916063a535520a7c6283740eaf218cd6961b10cba46fd"`,
    );
    const { skApp: appNullifierSecretKey2 } = await keyStore.getKeyValidationRequest(
      newComputedMasterNullifierPublicKeyHashes[2],
      appAddress,
    );
    expect(appNullifierSecretKey2.toString()).toMatchInlineSnapshot(
      `"0x27ccbe41ff5f33fa78348533da9d4a79e8fea8805771e61748ea42be4202f168"`,
    );

    expect(appNullifierSecretKey0).toEqual(computeAppNullifierSecretKey(newMasterNullifierSecretKeys[0], appAddress));
    expect(appNullifierSecretKey1).toEqual(computeAppNullifierSecretKey(newMasterNullifierSecretKeys[1], appAddress));
    expect(appNullifierSecretKey2).toEqual(computeAppNullifierSecretKey(newMasterNullifierSecretKeys[2], appAddress));
  });
});
