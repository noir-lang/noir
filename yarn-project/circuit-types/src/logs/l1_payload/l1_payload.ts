import {
  type AztecAddress,
  type GrumpkinScalar,
  type KeyValidationRequest,
  type PublicKey,
  computeOvskApp,
  derivePublicKeyFromSecretKey,
} from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { type Fr, Point } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { EncryptedLogHeader } from './encrypted_log_header.js';
import { type EncryptedLogIncomingBody } from './encrypted_log_incoming_body/index.js';
import { EncryptedLogOutgoingBody } from './encrypted_log_outgoing_body.js';

// Both the incoming and the outgoing header are 48 bytes.
// 32 bytes for the address, and 16 bytes padding to follow PKCS#7
const HEADER_SIZE = 48;

// The outgoing body is constant size of 144 bytes.
// 128 bytes for the secret key, address and public key, and 16 bytes padding to follow PKCS#7
const OUTGOING_BODY_SIZE = 144;
/**
 * A class which wraps event data which is pushed on L1.
 */
export abstract class L1Payload {
  /**
   * Serializes the L1EventPayload object into a Buffer.
   * @returns Buffer representation of the L1EventPayload object.
   */
  abstract toBuffer(): Buffer;

  /**
   * Encrypts an event payload for a given recipient and sender.
   * Creates an incoming log the the recipient using the recipient's ivsk, and
   * an outgoing log for the sender using the sender's ovsk.
   *
   * @param ephSk - An ephemeral secret key used for the encryption
   * @param recipient - The recipient address, retrievable by the sender for his logs
   * @param ivpk - The incoming viewing public key of the recipient
   * @param ovKeys - The outgoing viewing keys of the sender
   * @returns A buffer containing the encrypted log payload
   * @throws If the ivpk is zero.
   */
  protected _encrypt<T extends EncryptedLogIncomingBody>(
    contractAddress: AztecAddress,
    ephSk: GrumpkinScalar,
    recipient: AztecAddress,
    ivpk: PublicKey,
    ovKeys: KeyValidationRequest,
    incomingBody: T,
  ) {
    if (ivpk.isZero()) {
      throw new Error(`Attempting to encrypt an event log with a zero ivpk.`);
    }

    const ephPk = derivePublicKeyFromSecretKey(ephSk);

    const header = new EncryptedLogHeader(contractAddress);

    const incomingHeaderCiphertext = header.computeCiphertext(ephSk, ivpk);
    const outgoingHeaderCiphertext = header.computeCiphertext(ephSk, ovKeys.pkM);

    const incomingBodyCiphertext = incomingBody.computeCiphertext(ephSk, ivpk);

    const outgoingBodyCiphertext = new EncryptedLogOutgoingBody(ephSk, recipient, ivpk).computeCiphertext(
      ovKeys.skAppAsGrumpkinScalar,
      ephPk,
    );

    return Buffer.concat([
      ephPk.toCompressedBuffer(),
      incomingHeaderCiphertext,
      outgoingHeaderCiphertext,
      outgoingBodyCiphertext,
      incomingBodyCiphertext,
    ]);
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
   */
  protected static _decryptAsIncoming<T extends EncryptedLogIncomingBody>(
    data: Buffer,
    ivsk: GrumpkinScalar,
    fromCiphertext: (incomingBodySlice: Buffer, ivsk: GrumpkinScalar, ephPk: Point) => T,
  ): [AztecAddress, T] {
    const reader = BufferReader.asReader(data);

    const ephPk = Point.fromCompressedBuffer(reader.readBytes(Point.COMPRESSED_SIZE_IN_BYTES));

    const incomingHeader = EncryptedLogHeader.fromCiphertext(reader.readBytes(HEADER_SIZE), ivsk, ephPk);

    // Skipping the outgoing header and body
    reader.readBytes(HEADER_SIZE);
    reader.readBytes(OUTGOING_BODY_SIZE);

    // The incoming can be of variable size, so we read until the end
    const incomingBodySlice = reader.readToEnd();

    const incomingBody = fromCiphertext(incomingBodySlice, ivsk, ephPk);

    return [incomingHeader.address, incomingBody];
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
  protected static _decryptAsOutgoing<T extends EncryptedLogIncomingBody>(
    data: Buffer,
    ovsk: GrumpkinScalar,
    fromCiphertext: (incomingBodySlice: Buffer, ivsk: GrumpkinScalar, ephPk: Point) => T,
  ): [AztecAddress, T] {
    const reader = BufferReader.asReader(data);

    const ephPk = Point.fromCompressedBuffer(reader.readBytes(Point.COMPRESSED_SIZE_IN_BYTES));

    reader.readBytes(HEADER_SIZE);

    const outgoingHeader = EncryptedLogHeader.fromCiphertext(reader.readBytes(HEADER_SIZE), ovsk, ephPk);

    const ovskApp = computeOvskApp(ovsk, outgoingHeader.address);
    const outgoingBody = EncryptedLogOutgoingBody.fromCiphertext(reader.readBytes(OUTGOING_BODY_SIZE), ovskApp, ephPk);

    // The incoming can be of variable size, so we read until the end
    const incomingBodySlice = reader.readToEnd();

    const incomingBody = fromCiphertext(incomingBodySlice, outgoingBody.ephSk, outgoingBody.recipientIvpk);

    return [outgoingHeader.address, incomingBody];
  }

  protected static ensureMatchedMaskedContractAddress(
    contractAddress: AztecAddress,
    randomness: Fr,
    maskedContractAddress: Fr,
  ) {
    if (!pedersenHash([contractAddress, randomness], 0).equals(maskedContractAddress)) {
      throw new Error(
        'The provided masked contract address does not match with the incoming address from header and randomness from body',
      );
    }
  }
}
