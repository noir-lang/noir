export class Timer {
  private start: number;

  constructor() {
    this.start = new Date().getTime();
  }

  public ms() {
    return new Date().getTime() - this.start;
  }

  public s() {
    return (new Date().getTime() - this.start) / 1000;
  }
}
