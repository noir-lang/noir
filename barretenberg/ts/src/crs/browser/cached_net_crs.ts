import { NetCrs } from '../net_crs.js';
import { get, set } from 'idb-keyval';

/**
 * Downloader for CRS from the web or local.
 */
export class CachedNetCrs {
  private g1Data!: Uint8Array;
  private g2Data!: Uint8Array;

  constructor(public readonly numPoints: number) {}

  static async new(numPoints: number) {
    const crs = new CachedNetCrs(numPoints);
    await crs.init();
    return crs;
  }

  /**
   * Download the data.
   */
  async init() {
    // Check if data is in IndexedDB
    const g1Data = await get('g1Data');
    const g2Data = await get('g2Data');
    const netCrs = new NetCrs(this.numPoints);
    const g1DataLength = this.numPoints * 64;

    if (!g1Data || g1Data.length < g1DataLength) {
      this.g1Data = await netCrs.downloadG1Data();
      await set('g1Data', this.g1Data);
    } else {
      this.g1Data = g1Data;
    }

    if (!g2Data) {
      this.g2Data = await netCrs.downloadG2Data();
      await set('g2Data', this.g2Data);
    } else {
      this.g2Data = g2Data;
    }
  }

  /**
   * G1 points data for prover key.
   * @returns The points data.
   */
  getG1Data(): Uint8Array {
    return this.g1Data;
  }

  /**
   * G2 points data for verification key.
   * @returns The points data.
   */
  getG2Data(): Uint8Array {
    return this.g2Data;
  }
}
