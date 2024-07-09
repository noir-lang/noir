import { AztecAddress, type GrumpkinScalar, type KeyValidationRequest, type PublicKey } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { type EncryptedL2Log } from '../encrypted_l2_log.js';
import { EncryptedEventLogIncomingBody } from './encrypted_log_incoming_body/index.js';
import { L1Payload } from './l1_payload.js';
import { Event } from './payload.js';

/**
 * A class which wraps event data which is pushed on L1.
 */
export class L1EventPayload extends L1Payload {
  constructor(
    /**
     * An encrypted event as emitted from Noir contract.
     */
    public event: Event,
    /**
     * Address of the contract that emitted this event log.
     */
    public contractAddress: AztecAddress,
    /**
     * Randomness used to mask the contract address.
     */
    public randomness: Fr,
    /**
     * Type identifier for the underlying event.
     */
    public eventTypeId: EventSelector,
  ) {
    super();
  }

  /**
   * Deserializes the L1EventPayload object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of L1EventPayload.
   */
  static fromBuffer(buffer: Buffer | BufferReader): L1EventPayload {
    const reader = BufferReader.asReader(buffer);
    return new L1EventPayload(
      reader.readObject(Event),
      reader.readObject(AztecAddress),
      Fr.fromBuffer(reader),
      reader.readObject(EventSelector),
    );
  }

  /**
   * Serializes the L1EventPayload object into a Buffer.
   * @returns Buffer representation of the L1EventPayload object.
   */
  toBuffer() {
    return serializeToBuffer([this.event, this.contractAddress, this.randomness, this.eventTypeId]);
  }

  /**
   * Create a random L1EventPayload object (useful for testing purposes).
   * @returns A random L1EventPayload object.
   */
  static random() {
    return new L1EventPayload(Event.random(), AztecAddress.random(), Fr.random(), EventSelector.random());
  }

  public encrypt(ephSk: GrumpkinScalar, recipient: AztecAddress, ivpk: PublicKey, ovKeys: KeyValidationRequest) {
    return super._encrypt(
      this.contractAddress,
      ephSk,
      recipient,
      ivpk,
      ovKeys,
      new EncryptedEventLogIncomingBody(this.randomness, this.eventTypeId.toField(), this.event),
    );
  }

  /**
   * Decrypts a ciphertext as an incoming log.
   *
   * This is executable by the recipient of the event, and uses the ivsk to decrypt the payload.
   * The outgoing parts of the log are ignored entirely.
   *
   * Produces the same output as `decryptAsOutgoing`.
   *
   * @param encryptedLog - The encrypted log. This encrypted log is assumed to always have tags.
   * @param ivsk - The incoming viewing secret key, used to decrypt the logs
   * @returns The decrypted log payload
   * @remarks The encrypted log is assumed to always have tags.
   */
  public static decryptAsIncoming(encryptedLog: EncryptedL2Log, ivsk: GrumpkinScalar) {
    const reader = BufferReader.asReader(encryptedLog.data);

    // We skip the tags
    Fr.fromBuffer(reader);
    Fr.fromBuffer(reader);

    const [address, incomingBody] = super._decryptAsIncoming(
      reader.readToEnd(),
      ivsk,
      EncryptedEventLogIncomingBody.fromCiphertext,
    );

    // We instantiate selector before checking the address because instantiating the selector validates that
    // the selector is valid (and that's the preferred way of detecting decryption failure).
    const selector = EventSelector.fromField(incomingBody.eventTypeId);

    this.ensureMatchedMaskedContractAddress(address, incomingBody.randomness, encryptedLog.maskedContractAddress);

    return new L1EventPayload(incomingBody.event, address, incomingBody.randomness, selector);
  }

  /**
   * Decrypts a ciphertext as an outgoing log.
   *
   * This is executable by the sender of the event, and uses the ovsk to decrypt the payload.
   * The outgoing parts are decrypted to retrieve information that allows the sender to
   * decrypt the incoming log, and learn about the event contents.
   *
   * Produces the same output as `decryptAsIncoming`.
   *
   * @param ciphertext - The ciphertext for the log
   * @param ovsk - The outgoing viewing secret key, used to decrypt the logs
   * @returns The decrypted log payload
   */
  public static decryptAsOutgoing(encryptedLog: EncryptedL2Log, ovsk: GrumpkinScalar) {
    const reader = BufferReader.asReader(encryptedLog.data);

    // Skip the tags
    Fr.fromBuffer(reader);
    Fr.fromBuffer(reader);

    const [address, incomingBody] = super._decryptAsOutgoing(
      reader.readToEnd(),
      ovsk,
      EncryptedEventLogIncomingBody.fromCiphertext,
    );

    // We instantiate selector before checking the address because instantiating the selector validates that
    // the selector is valid (and that's the preferred way of detecting decryption failure).
    const selector = EventSelector.fromField(incomingBody.eventTypeId);

    this.ensureMatchedMaskedContractAddress(address, incomingBody.randomness, encryptedLog.maskedContractAddress);

    return new L1EventPayload(incomingBody.event, address, incomingBody.randomness, selector);
  }
}
