export interface BBConfig {
  bbBinaryPath: string;
  bbWorkingDirectory: string;
  /** Whether to skip tmp dir cleanup for debugging purposes */
  bbSkipCleanup?: boolean;
}

export interface ACVMConfig {
  acvmBinaryPath: string;
  acvmWorkingDirectory: string;
}
