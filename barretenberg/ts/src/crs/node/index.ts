import { NetCrs } from '../net_crs.js';
import { IgnitionFilesCrs } from './ignition_files_crs.js';
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

    const crs = IgnitionFilesCrs.defaultExists() ? new IgnitionFilesCrs(this.numPoints) : new NetCrs(this.numPoints);
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
