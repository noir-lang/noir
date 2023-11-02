import { parse as commandLineArgs } from 'ts-command-line-args';

const DEFAULT_GLOB_PATTERN = './target/**/*.json';

export interface ParsedArgs {
  files: string[];
  outDir?: string | undefined;
  inputDir?: string | undefined;
}

export function parseArgs(): ParsedArgs {
  const rawOptions = commandLineArgs<CommandLineArgs>(
    {
      glob: {
        type: String,
        defaultOption: true,
        multiple: true,
        defaultValue: [DEFAULT_GLOB_PATTERN],
        description:
          'Pattern that will be used to find program artifacts. Remember about adding quotes: noir-codegen "**/*.json".',
      },
      'out-dir': { type: String, optional: true, description: 'Output directory for generated files.' },
      'input-dir': {
        type: String,
        optional: true,
        description:
          'Directory containing program artifact files. Inferred as lowest common path of all files if not specified.',
      },
      help: { type: Boolean, defaultValue: false, alias: 'h', description: 'Prints this message.' },
    },
    {
      helpArg: 'help',
      headerContentSections: [
        {
          content: `\
          noir-codegen generates TypeScript wrappers for Noir programs to simplify replicating your Noir logic in JS.`,
        },
      ],
      footerContentSections: [
        {
          header: 'Example Usage',
          content: `\
          noir-codegen --out-dir app/noir_programs './target/*.json'


          You can read more about noir-codegen at {underline https://github.com/noir-lang/noir}.`,
        },
      ],
    },
  );

  return {
    files: rawOptions.glob,
    outDir: rawOptions['out-dir'],
    inputDir: rawOptions['input-dir'],
  };
}

interface CommandLineArgs {
  glob: string[];
  'out-dir'?: string;
  'input-dir'?: string;
  help: boolean;
}
