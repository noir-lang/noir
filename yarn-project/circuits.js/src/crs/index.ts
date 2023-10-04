import { createDebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';

import isNode from 'detect-node';
import { existsSync } from 'fs';
import { open } from 'fs/promises';
import { dirname, join } from 'path';

/**
 * Downloader for CRS from the web or local.
 */
export class NetCrs {
  private data!: Uint8Array;
  private g2Data!: Uint8Array;

  constructor(
    /**
     * The number of circuit gates.
     */
    public readonly numPoints: number,
  ) {}

  /**
   * Download the data.
   */
  async init() {
    // We need numPoints number of g1 points.
    const g1Start = 28;
    const g1End = g1Start + this.numPoints * 64 - 1;
    // Download required range of data.
    const response = await fetch('https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/monomial/transcript00.dat', {
      headers: {
        Range: `bytes=${g1Start}-${g1End}`,
      },
    });

    this.data = new Uint8Array(await response.arrayBuffer());

    await this.downloadG2Data();
  }

  /**
   * Download the G2 points data.
   */
  async downloadG2Data() {
    const g2Start = 28 + 5_040_001 * 64; // = 322_560_092

    const g2End = g2Start + 128 - 1;

    const response2 = await fetch('https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/monomial/transcript00.dat', {
      headers: {
        Range: `bytes=${g2Start}-${g2End}`,
      },
    });

    this.g2Data = new Uint8Array(await response2.arrayBuffer());
  }

  /**
   * Verification key data.
   * @returns The verification key.
   */
  getG1Data(): Uint8Array {
    return this.data;
  }

  /**
   * G2 points data.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return this.g2Data;
  }
}

/**
 * Downloader for CRS from a local file (for Node).
 */
export class FileCrs {
  private data!: Uint8Array;
  private g2Data!: Uint8Array;

  constructor(
    /**
     * The number of circuit gates.
     */
    public readonly numPoints: number,
    private path: string,
    private readonly offsetStart = true,
  ) {}

  /**
   * Read the data file.
   */
  async init() {
    // We need numPoints number of g1 points.
    const g1Start = this.offsetStart ? 28 : 0;
    const g1Length = this.numPoints * 64;

    const g2Start = 28 + 5_040_001 * 64; // = 322_560_092
    const g2Length = 128;
    // Lazily seek our data
    const fileHandle = await open(this.path, 'r');
    try {
      this.data = Buffer.alloc(g1Length);
      await fileHandle.read(this.data, 0, g1Length, g1Start);

      this.g2Data = Buffer.alloc(g2Length);
      await fileHandle.read(this.g2Data, 0, g2Length, this.offsetStart ? g2Start : g1Start + g1Length);
    } finally {
      await fileHandle.close();
    }
  }

  /**
   * Verification key data.
   * @returns The verification key.
   */
  getG1Data(): Uint8Array {
    return this.data;
  }

  /**
   * G2 points data.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return this.g2Data;
  }
}

/**
 * Generic CRS finder utility class.
 */
export class Crs {
  private crs: FileCrs | NetCrs;
  private logger = createDebugLogger('circuits:crs');
  /**
   * The path to our SRS object, assuming that we are in the aztec3-packages folder structure.
   */
  private devPath = '/../../../../barretenberg/cpp/srs_db/ignition/monomial/transcript00.dat';
  /**
   * The path of our SRS object, if we downloaded on init.
   */
  private localPath = `/../../resources/ignition/monomial/transcript00.dat`;

  constructor(
    /**
     * The number of circuit gates.
     */
    public readonly numPoints: number,

    /**
     * Option to save downloaded SRS on file.
     */
    private readonly saveOnFile = true,
  ) {
    if (isNode) {
      const devPath = join(fileURLToPath(import.meta.url), this.devPath);
      const localPath = join(dirname(fileURLToPath(import.meta.url)), this.localPath);
      const existsDev = existsSync(devPath);
      const existsLocal = existsSync(localPath);

      if (existsDev) {
        this.crs = new FileCrs(numPoints, devPath);
      } else if (existsLocal) {
        this.crs = new FileCrs(numPoints, localPath, false);
      } else {
        this.crs = new NetCrs(numPoints);
      }
    } else {
      this.crs = new NetCrs(numPoints);
    }
  }

  /**
   * Read CRS from our chosen source.
   */
  async init() {
    await this.crs.init();
    if (isNode) {
      const localPath = dirname(fileURLToPath(import.meta.url)) + this.localPath;
      // save downloaded CRS on file
      if (this.saveOnFile && !existsSync(localPath)) {
        const fileHandle = await open(localPath, 'w');
        const g1Data = Buffer.from(this.crs.getG1Data());
        try {
          await fileHandle.write(g1Data);
        } catch (err: any) {
          this.logger.warn('Failed to save CRS data: ', err.message);
        }

        const g2Data = Buffer.from(this.crs.getG2Data());
        try {
          await fileHandle.write(g2Data, 0, g2Data.length, g1Data.length);
          // appendFileSync(localPath, Buffer.from(g2Data));
        } catch (err: any) {
          this.logger.warn('Failed to append G2 data: ', err.message);
        } finally {
          await fileHandle.close();
        }
      }
    }
  }

  /**
   * Verification key data.
   * @returns The verification key.
   */
  getG1Data(): Uint8Array {
    return this.crs.getG1Data();
  }

  /**
   * G2 points data.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return this.crs.getG2Data();
  }
}
