import { type AccountWallet, AztecAddress, Fr, type PXE } from '@aztec/aztec.js';
import { CompleteAddress, GeneratorIndex, type PartialAddress, Point } from '@aztec/circuits.js';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { KeyRegistryContract, TestContract } from '@aztec/noir-contracts.js';
import { getCanonicalKeyRegistryAddress } from '@aztec/protocol-contracts/key-registry';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('Key Registry', () => {
  let keyRegistry: KeyRegistryContract;

  let pxe: PXE;
  let testContract: TestContract;
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWallet[];

  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, pxe, wallets } = await setup(3));
    keyRegistry = await KeyRegistryContract.at(getCanonicalKeyRegistryAddress(), wallets[0]);

    testContract = await TestContract.deploy(wallets[0]).send().deployed();

    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));
  });

  const delay = async (blocks: number) => {
    for (let i = 0; i < blocks; i++) {
      await testContract.methods.delay().send().wait();
    }
  };

  afterAll(() => teardown());

  describe('failure cases', () => {
    let accountAddedToRegistry: AztecAddress;

    describe('should fail when registering with different types of invalid input', () => {
      const masterNullifierPublicKey = Point.random();
      const masterIncomingViewingPublicKey = Point.random();
      const masterOutgoingViewingPublicKey = Point.random();
      const masterTaggingPublicKey = Point.random();
      const partialAddress: PartialAddress = Fr.random();

      // TODO(#5726): use computePublicKeysHash function
      const publicKeysHash = poseidon2Hash([
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
        GeneratorIndex.PUBLIC_KEYS_HASH,
      ]);

      // TODO(#5726): Move the following line to AztecAddress class?
      accountAddedToRegistry = AztecAddress.fromField(
        poseidon2Hash([publicKeysHash, partialAddress, GeneratorIndex.CONTRACT_ADDRESS_V1]),
      );

      it('should fail when we register with a mismatched address', async () => {
        const mismatchedAddress = AztecAddress.random();

        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.register(
              mismatchedAddress,
              partialAddress,
              masterNullifierPublicKey,
              masterIncomingViewingPublicKey,
              masterOutgoingViewingPublicKey,
              masterTaggingPublicKey,
            )
            .send()
            .wait(),
        ).rejects.toThrow('Computed address does not match supplied address');
      });

      it('should fail when we register with mismatched nullifier public key', async () => {
        const mismatchedMasterNullifierPublicKey = Point.random();

        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.register(
              AztecAddress.fromField(accountAddedToRegistry),
              partialAddress,
              mismatchedMasterNullifierPublicKey,
              masterIncomingViewingPublicKey,
              masterOutgoingViewingPublicKey,
              masterTaggingPublicKey,
            )
            .send()
            .wait(),
        ).rejects.toThrow('Computed address does not match supplied address');
      });
    });

    describe('should fail when rotating keys with different types of bad input', () => {
      it('should fail when we try to rotate keys, while setting a 0 key', async () => {
        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.rotate_nullifier_public_key(wallets[0].getAddress(), Point.ZERO, Fr.ZERO)
            .send()
            .wait(),
        ).rejects.toThrow('New nullifier public key must be non-zero');
      });

      it('should fail when we try to rotate keys for another address without authwit', async () => {
        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.rotate_nullifier_public_key(wallets[1].getAddress(), Point.random(), Fr.ZERO)
            .send()
            .wait(),
        ).rejects.toThrow('Assertion failed: Message not authorized by account');
      });
    });
  });

  describe('key registration flow', () => {
    let accountAddedToRegistry: AztecAddress;
    const masterNullifierPublicKey = Point.random();

    it('should generate master public keys, a partial address, and register with the key registry', async () => {
      const masterIncomingViewingPublicKey = Point.random();
      const masterOutgoingViewingPublicKey = Point.random();
      const masterTaggingPublicKey = Point.random();
      const partialAddress: PartialAddress = new Fr(420);

      const publicKeysHash = poseidon2Hash([
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
        GeneratorIndex.PUBLIC_KEYS_HASH,
      ]);

      // TODO(#5726): Move the following line to AztecAddress class?
      accountAddedToRegistry = AztecAddress.fromField(
        poseidon2Hash([publicKeysHash, partialAddress, GeneratorIndex.CONTRACT_ADDRESS_V1]),
      );

      await keyRegistry
        .withWallet(wallets[0])
        .methods.register(
          AztecAddress.fromField(accountAddedToRegistry),
          partialAddress,
          masterNullifierPublicKey,
          masterIncomingViewingPublicKey,
          masterOutgoingViewingPublicKey,
          masterTaggingPublicKey,
        )
        .send()
        .wait();

      // We check if our registered nullifier key is equal to the key obtained from the getter by
      // reading our registry contract from the test contract. We expect this to fail because the change has not been applied yet
      const emptyNullifierPublicKey = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(1, accountAddedToRegistry)
        .simulate();

      expect(new Fr(emptyNullifierPublicKey)).toEqual(Fr.ZERO);

      // We check it again after a delay and expect that the change has been applied and consequently the assert is true
      await delay(5);

      const nullifierPublicKey = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(1, accountAddedToRegistry)
        .simulate();

      expect(new Fr(nullifierPublicKey)).toEqual(poseidon2Hash(masterNullifierPublicKey.toFields()));
    });
  });

  describe('key rotation flows', () => {
    const firstNewMasterNullifierPublicKey = Point.random();

    describe('key rotation flow without authwit', () => {
      it('we call the key registry to rotate our nullifier key', async () => {
        await keyRegistry
          .withWallet(wallets[0])
          .methods.rotate_nullifier_public_key(wallets[0].getAddress(), firstNewMasterNullifierPublicKey, Fr.ZERO)
          .send()
          .wait();

        // We check if our rotated nullifier key is equal to the key obtained from the getter by
        // reading our registry contract from the test contract. We expect this to fail because the change has not been applied yet
        const emptyNullifierPublicKey = await testContract.methods
          .test_shared_mutable_private_getter_for_registry_contract(1, wallets[0].getAddress())
          .simulate();

        expect(new Fr(emptyNullifierPublicKey)).toEqual(Fr.ZERO);

        // We check it again after a delay and expect that the change has been applied and consequently the assert is true
        await delay(5);

        const nullifierPublicKey = await testContract.methods
          .test_shared_mutable_private_getter_for_registry_contract(1, wallets[0].getAddress())
          .simulate();

        expect(new Fr(nullifierPublicKey)).toEqual(poseidon2Hash(firstNewMasterNullifierPublicKey.toFields()));
      });
    });

    describe('key rotation flow with authwit', () => {
      const secondNewMasterNullifierPublicKey = Point.random();

      it(`wallet 1 rotates wallet 0's nullifying public key with an authwit`, async () => {
        const action = keyRegistry
          .withWallet(wallets[1])
          .methods.rotate_nullifier_public_key(wallets[0].getAddress(), secondNewMasterNullifierPublicKey, Fr.ZERO);

        await wallets[0]
          .setPublicAuthWit({ caller: wallets[1].getCompleteAddress().address, action }, true)
          .send()
          .wait();

        await action.send().wait();

        // We check if our rotated nullifier key is equal to the key obtained from the getter by
        // reading our registry contract from the test contract. We expect this value to be the old one, because the new one hasn't been applied
        const oldNullifierPublicKey = await testContract.methods
          .test_shared_mutable_private_getter_for_registry_contract(1, wallets[0].getAddress())
          .simulate();

        expect(new Fr(oldNullifierPublicKey)).toEqual(poseidon2Hash(firstNewMasterNullifierPublicKey.toFields()));

        // We check it again after a delay and expect that the change has been applied and consequently the assert is true
        await delay(5);

        const newNullifierPublicKey = await testContract.methods
          .test_shared_mutable_private_getter_for_registry_contract(1, wallets[0].getAddress())
          .simulate();

        expect(new Fr(newNullifierPublicKey)).toEqual(poseidon2Hash(secondNewMasterNullifierPublicKey.toFields()));
      });
    });
  });

  describe('testing get_fresh_nullifier_public_key_hash: key registration flow, no PXE', () => {
    const masterNullifierPublicKey = Point.random();
    const masterIncomingViewingPublicKey = Point.random();
    const masterOutgoingViewingPublicKey = Point.random();
    const masterTaggingPublicKey = Point.random();
    const partialAddress: PartialAddress = new Fr(420);

    const publicKeysHash = poseidon2Hash([
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      GeneratorIndex.PUBLIC_KEYS_HASH,
    ]);

    // TODO(#5726): Move the following line to AztecAddress class?
    const accountAddedToRegistry = AztecAddress.fromField(
      poseidon2Hash([publicKeysHash, partialAddress, GeneratorIndex.CONTRACT_ADDRESS_V1]),
    );

    it('should fail as we have not registered anything to the registry nor have we registered a recipient', async () => {
      await expect(
        testContract.methods
          .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
          .send()
          .wait(),
      ).rejects.toThrow(`Cannot satisfy constraint 'computed_address.eq(address)'`);
    });

    it('adds an entry to the key registry, and checks the key freshness without and with conflicting information from our pxe', async () => {
      await keyRegistry
        .withWallet(wallets[0])
        .methods.register(
          AztecAddress.fromField(accountAddedToRegistry),
          partialAddress,
          masterNullifierPublicKey,
          masterIncomingViewingPublicKey,
          masterOutgoingViewingPublicKey,
          masterTaggingPublicKey,
        )
        .send()
        .wait();

      // We check if our registered nullifier key is equal to the key obtained from the getter by
      // reading our registry contract from the test contract. We expect this to fail because the change has not been applied yet
      await expect(
        testContract.methods
          .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
          .send()
          .wait(),
      ).rejects.toThrow(`Cannot satisfy constraint 'computed_address.eq(address)'`);

      // We check it again after a delay and expect that the change has been applied and consequently the assert is true
      await delay(5);

      await testContract.methods
        .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
        .send()
        .wait();

      // TODO: (#5834) Refactor complete address to move the public keys
      await pxe.registerRecipient(CompleteAddress.create(accountAddedToRegistry, Point.ZERO, partialAddress), [
        new Point(Fr.random(), Fr.random()),
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
      ]);

      // Our check should still succeed even if our pxe gives conflicting information, taking the registry as the source of truth.
      await testContract.methods
        .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
        .send()
        .wait();
    });
  });

  describe('testing assert_nullifier_key_is_fresh: key registration flow, with PXE', () => {
    const masterNullifierPublicKey = Point.random();
    const masterIncomingViewingPublicKey = Point.random();
    const masterOutgoingViewingPublicKey = Point.random();
    const masterTaggingPublicKey = Point.random();
    const partialAddress: PartialAddress = new Fr(69420);

    const publicKeysHash = poseidon2Hash([
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      GeneratorIndex.PUBLIC_KEYS_HASH,
    ]);

    // TODO(#5726): Move the following line to AztecAddress class?
    const accountAddedToRegistry = AztecAddress.fromField(
      poseidon2Hash([publicKeysHash, partialAddress, GeneratorIndex.CONTRACT_ADDRESS_V1]),
    );

    it('should fail as we have not registered anything to the registry nor have we registered a recipient', async () => {
      await expect(
        testContract.methods
          .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
          .send()
          .wait(),
      ).rejects.toThrow(`Cannot satisfy constraint 'computed_address.eq(address)'`);
    });

    it('should fail when we try to check the public keys for a invalid address', async () => {
      const randAddress = AztecAddress.random();
      // TODO: (#5834) Refactor complete address to move the public keys
      await pxe.registerRecipient(CompleteAddress.create(randAddress, Point.ZERO, partialAddress), [
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
      ]);

      await expect(
        testContract.methods.test_nullifier_key_freshness(randAddress, masterNullifierPublicKey).send().wait(),
      ).rejects.toThrow(`Cannot satisfy constraint 'computed_address.eq(address)'`);
    });

    it('adds a recipient to our pxe, and checks the key freshness with and without adding an entry to our key registry', async () => {
      // TODO: (#5834) Refactor complete address to move the public keys
      await pxe.registerRecipient(CompleteAddress.create(accountAddedToRegistry, Point.ZERO, partialAddress), [
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
      ]);

      // The check should succeed because we register our recipient manually and the lib checks our pxe
      await testContract.methods
        .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
        .send()
        .wait();

      // Now we add the keys to registry
      await keyRegistry
        .withWallet(wallets[0])
        .methods.register(
          AztecAddress.fromField(accountAddedToRegistry),
          partialAddress,
          masterNullifierPublicKey,
          masterIncomingViewingPublicKey,
          masterOutgoingViewingPublicKey,
          masterTaggingPublicKey,
        )
        .send()
        .wait();

      // We check if our rotated nullifier key is equal to the key obtained from the getter by
      // reading our registry contract from the test contract. We expect this to be 0 because the change has not been applied yet
      const emptyNullifierPublicKey = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(1, accountAddedToRegistry)
        .simulate();

      expect(new Fr(emptyNullifierPublicKey)).toEqual(Fr.ZERO);

      // We check if our rotated nullifier key is equal to the key obtained from the getter. We expect this to succeed because even though the change
      // has not been applied yet to the registry, we have manually the keys to our pxe
      await testContract.methods
        .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
        .send()
        .wait();

      // In the case where the key exists both in the pxe and our registry, we know that our assert will still remain true
      await testContract.methods
        .test_nullifier_key_freshness(accountAddedToRegistry, masterNullifierPublicKey)
        .send()
        .wait();
    });
  });

  describe('testing assert_nullifier_key_is_fresh: key rotation flow', () => {
    const newMasterNullifierPublicKey = Point.random();

    it('we rotate the nullifier key and check that the key is fresh', async () => {
      await keyRegistry
        .withWallet(wallets[0])
        .methods.rotate_nullifier_public_key(wallets[0].getAddress(), newMasterNullifierPublicKey, Fr.ZERO)
        .send()
        .wait();

      // We check if our rotated nullifier key is equal to the key obtained from the getter by
      // reading our registry contract from the test contract. We expect this to fail because the change has not been applied yet
      await expect(
        testContract.methods
          .test_nullifier_key_freshness(wallets[0].getAddress(), newMasterNullifierPublicKey)
          .send()
          .wait(),
      ).rejects.toThrow(
        `Cannot satisfy constraint 'assert_eq(get_fresh_nullifier_public_key_hash(&mut context, address), poseidon2_hash(public_nullifying_key.serialize()))'`,
      );

      // We check it again after a delay and expect that the change has been applied and consequently the assert is true
      await delay(5);

      await testContract.methods
        .test_nullifier_key_freshness(wallets[0].getAddress(), newMasterNullifierPublicKey)
        .send()
        .wait();
    });
  });
});
