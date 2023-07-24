export interface Arg {
  name: string;
  type: string;
}

export interface FunctionDeclaration {
  functionName: string;
  inArgs: Arg[];
  outArgs: Arg[];
  isAsync: boolean;
}
