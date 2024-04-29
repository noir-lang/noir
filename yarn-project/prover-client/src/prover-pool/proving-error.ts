export class ProvingError extends Error {
  override toString() {
    return this.message;
  }

  static fromString(message: string) {
    return new ProvingError(message);
  }
}
