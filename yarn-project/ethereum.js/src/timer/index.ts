// eslint-disable-next-line jsdoc/require-jsdoc
export class Timer {
  private start: number;

  constructor() {
    this.start = new Date().getTime();
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public ms() {
    return new Date().getTime() - this.start;
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public s() {
    return (new Date().getTime() - this.start) / 1000;
  }
}
