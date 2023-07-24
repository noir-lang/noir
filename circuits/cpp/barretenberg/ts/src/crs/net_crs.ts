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
    await this.downloadG1Data();
    await this.downloadG2Data();
  }

  async downloadG1Data() {
    const g1Start = 28;
    const g1End = g1Start + this.numPoints * 64 - 1;

    const response = await fetch('https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/monomial/transcript00.dat', {
      headers: {
        Range: `bytes=${g1Start}-${g1End}`,
      },
      cache: 'force-cache',
    });

    return (this.data = new Uint8Array(await response.arrayBuffer()));
  }

  /**
   * Download the G2 points data.
   */
  async downloadG2Data() {
    const g2Start = 28 + 5040000 * 64;
    const g2End = g2Start + 128 - 1;

    const response2 = await fetch('https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/sealed/transcript00.dat', {
      headers: {
        Range: `bytes=${g2Start}-${g2End}`,
      },
      cache: 'force-cache',
    });

    return (this.g2Data = new Uint8Array(await response2.arrayBuffer()));
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
