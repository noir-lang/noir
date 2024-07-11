import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import * as fs from 'fs/promises';
import path from 'path';

/**
 * TODO(https://github.com/AztecProtocol/aztec-packages/issues/7370) refactory this to
 * eventually we read all these VKs from the data tree instead of passing them
 */
export class ClientIvcProof {
  constructor(
    // produced by the sequencer when making the tube proof
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/7370): Need to precompute private kernel tail VK so we can verify this immediately in the tx pool
    // which parts of these are needed to quickly verify that we have a correct IVC proof
    public instVkBuffer: Buffer,
    public pgAccBuffer: Buffer,
    public clientIvcProofBuffer: Buffer,
    public translatorVkBuffer: Buffer,
    public eccVkBuffer: Buffer,
  ) {}

  public isEmpty() {
    return this.clientIvcProofBuffer.length === 0;
  }

  static empty() {
    return new ClientIvcProof(Buffer.from(''), Buffer.from(''), Buffer.from(''), Buffer.from(''), Buffer.from(''));
  }

  /**
   * TODO(#7371): eventually remove client_ivc_prove_output_all_msgpack and properly handle these accumulators and VKs
   * Create a ClientIvcProof from the result of client_ivc_prove_output_all or client_ivc_prove_output_all_msgpack
   * @param directory the directory of results
   * @returns the encapsulated client ivc proof
   */
  static async readFromOutputDirectory(directory: string) {
    const [instVkBuffer, pgAccBuffer, clientIvcProofBuffer, translatorVkBuffer, eccVkBuffer] = await Promise.all(
      ['inst_vk', 'pg_acc', 'client_ivc_proof', 'translator_vk', 'ecc_vk'].map(fileName =>
        fs.readFile(path.join(directory, fileName)),
      ),
    );
    return new ClientIvcProof(instVkBuffer, pgAccBuffer, clientIvcProofBuffer, translatorVkBuffer, eccVkBuffer);
  }

  /**
   * TODO(#7371): eventually remove client_ivc_prove_output_all_msgpack and properly handle these accumulators and VKs
   * Serialize a ClientIvcProof to the files expected by prove_tube
   *
   * Example usage:
   *  await runInDirectory(bbWorkingDirectory, async (dir: string) => {
   *    await privateTx.clientIvcProof!.writeToOutputDirectory(bbWorkingDirectory);
   *    const result = await generateTubeProof(bbPath, dir, logger.info)
   *    expect(result.status).toBe(BB_RESULT.SUCCESS)
   *  });
   * @param proof the ClientIvcProof from readFromOutputDirectory
   * @param directory the directory of results
   */
  async writeToOutputDirectory(directory: string) {
    const { instVkBuffer, pgAccBuffer, clientIvcProofBuffer, translatorVkBuffer, eccVkBuffer } = this;
    const fileData = [
      ['inst_vk', instVkBuffer],
      ['pg_acc', pgAccBuffer],
      ['client_ivc_proof', clientIvcProofBuffer],
      ['translator_vk', translatorVkBuffer],
      ['ecc_vk', eccVkBuffer],
    ] as const;
    await Promise.all(fileData.map(([fileName, buffer]) => fs.writeFile(path.join(directory, fileName), buffer)));
  }

  static fromBuffer(buffer: Buffer | BufferReader): ClientIvcProof {
    const reader = BufferReader.asReader(buffer);
    return new ClientIvcProof(
      reader.readBuffer(),
      reader.readBuffer(),
      reader.readBuffer(),
      reader.readBuffer(),
      reader.readBuffer(),
    );
  }

  public toBuffer() {
    return serializeToBuffer(
      this.instVkBuffer.length,
      this.instVkBuffer,
      this.pgAccBuffer.length,
      this.pgAccBuffer,
      this.clientIvcProofBuffer.length,
      this.clientIvcProofBuffer,
      this.translatorVkBuffer.length,
      this.translatorVkBuffer,
      this.eccVkBuffer.length,
      this.eccVkBuffer,
    );
  }
}
