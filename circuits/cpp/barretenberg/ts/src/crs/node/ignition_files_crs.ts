import { existsSync } from 'fs';
import { readFile } from 'fs/promises';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

/**
 * The path to our SRS object, assuming that we are in barretenberg/ts folder.
 */
export const SRS_DEV_PATH = dirname(fileURLToPath(import.meta.url)) + '/../../../cpp/srs_db/ignition/monomial';

/**
 * Downloader for CRS from a local file (for Node).
 */
export class IgnitionFilesCrs {
  private data!: Uint8Array;
  private g2Data!: Uint8Array;

  constructor(
    /**
     * The number of circuit gates.
     */
    public readonly numPoints: number,
    private path = SRS_DEV_PATH,
  ) {}

  static defaultExists() {
    return existsSync(SRS_DEV_PATH);
  }

  /**
   * Read the data file.
   */
  async init() {
    // We need this.numPoints number of g1 points.
    // numPoints should be circuitSize + 1.
    const g1Start = 28;
    const g1End = g1Start + this.numPoints * 64;

    const data = await readFile(this.path + '/transcript00.dat');
    this.data = data.subarray(g1Start, g1End);

    this.g2Data = await readFile(this.path + '/g2.dat');
  }

  /**
   * G1 points data for prover key.
   * @returns The points data.
   */
  getG1Data(): Uint8Array {
    return this.data;
  }

  /**
   * G2 points data for verification key.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return this.g2Data;
  }
}
