import { NetCrs } from '../net_crs.js';
import { GRUMPKIN_SRS_DEV_PATH, IgnitionFilesCrs } from './ignition_files_crs.js';
import { mkdirSync, readFileSync, writeFileSync } from 'fs';
import { readFile } from 'fs/promises';
import createDebug from 'debug';

const debug = createDebug('bb.js:crs');

/**
 * Generic CRS finder utility class.
 */
export class Crs {
  constructor(public readonly numPoints: number, public readonly path: string) {}

  static async new(numPoints: number, crsPath = './crs') {
    const crs = new Crs(numPoints, crsPath);
    await crs.init();
    return crs;
  }

  async init() {
    mkdirSync(this.path, { recursive: true });
    const size = await readFile(this.path + '/size', 'ascii').catch(() => undefined);
    if (size && +size >= this.numPoints) {
      debug(`using cached crs of size: ${size}`);
      return;
    }

    const ignitionCrs = new IgnitionFilesCrs(this.numPoints);
    const crs = ignitionCrs.pathExists() ? new IgnitionFilesCrs(this.numPoints) : new NetCrs(this.numPoints);
    if (crs instanceof NetCrs) {
      debug(`downloading crs of size: ${this.numPoints}`);
    } else {
      debug(`loading igntion file crs of size: ${this.numPoints}`);
    }
    await crs.init();
    writeFileSync(this.path + '/size', this.numPoints.toString());
    writeFileSync(this.path + '/g1.dat', crs.getG1Data());
    writeFileSync(this.path + '/g2.dat', crs.getG2Data());
  }

  /**
   * G1 points data for prover key.
   * @returns The points data.
   */
  getG1Data(): Uint8Array {
    return readFileSync(this.path + '/g1.dat');
  }

  /**
   * G2 points data for verification key.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return readFileSync(this.path + '/g2.dat');
  }
}

/**
 * Generic Grumpkin CRS finder utility class.
 */
export class GrumpkinCrs {
  constructor(public readonly numPoints: number, public readonly path: string) {}

  static async new(numPoints: number, crsPath = './crs') {
    const crs = new GrumpkinCrs(numPoints, crsPath);
    await crs.init();
    return crs;
  }

  async downloadG1Data() {
    const g1Start = 28;
    const g1End = g1Start + this.numPoints * 64 - 1;

    const response = await fetch('https://aztec-ignition.s3.amazonaws.com/TEST%20GRUMPKIN/monomial/transcript00.dat', {
      headers: {
        Range: `bytes=${g1Start}-${g1End}`,
      },
      cache: 'force-cache',
    });

    return new Uint8Array(await response.arrayBuffer());
  }

  async init() {
    mkdirSync(this.path, { recursive: true });
    const size = await readFile(this.path + '/grumpkin_size', 'ascii').catch(() => undefined);
    if (size && +size >= this.numPoints) {
      debug(`using cached crs of size: ${size}`);
      return;
    }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/813): implement NetCrs for Grumpkin once SRS is uploaded.
    const ignitionCrs = new IgnitionFilesCrs(this.numPoints, GRUMPKIN_SRS_DEV_PATH);
    if (ignitionCrs.pathExists()) {
      await ignitionCrs.init();
    }
    const g1Data = ignitionCrs.pathExists() ? ignitionCrs.getG1Data() : await this.downloadG1Data();
    debug(`loading ignition file crs of size: ${this.numPoints}`);
    // await crs.init();
    writeFileSync(this.path + '/grumpkin_size', this.numPoints.toString());
    writeFileSync(this.path + '/grumpkin_g1.dat', g1Data);
  }

  /**
   * G1 points data for prover key.
   * @returns The points data.
   */
  getG1Data(): Uint8Array {
    return readFileSync(this.path + '/grumpkin_g1.dat');
  }
}
