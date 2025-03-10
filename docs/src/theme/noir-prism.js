(function (Prism) {
  Prism.languages.noir = {
    comment: [
      {
        pattern: /\/\*[^]*?\*\//,
        greedy: true,
      },
      {
        pattern: /(^|[^\\:])\/\/.*/,
        lookbehind: true,
        greedy: true,
      },
    ],
    string: {
      pattern: /b?\"(?:\\[\s\S]|[^\\\"])*\"|b?r(#*)\"(?:[^"]|\"(?!\1))*\"\1/,
      greedy: true,
    },
    attribute: {
      pattern: /#!?\[(?:[^\[\]"]|\"(?:\\[\s\S]|[^\\\"])*\")*\]/,
      greedy: true,
      alias: 'attr-name',
      inside: {
        string: null, // see below
      },
    },

    'closure-params': {
      pattern: /([=(,:]\s*)\|[^|]*\||\|[^|]*\|(?=\s*(?:\{|->))/,
      lookbehind: true,
      greedy: true,
      inside: {
        'closure-punctuation': {
          pattern: /^\||\|$/,
          alias: 'punctuation',
        },
        rest: null, // see below
      },
    },

    'function-definition': {
      pattern: /(\bfn\s+)\w+/,
      lookbehind: true,
      alias: 'function',
    },
    'type-definition': {
      pattern: /(\b(?:enum|struct|trait|type)\s+)\w+/,
      lookbehind: true,
      alias: 'class-name',
    },
    'module-declaration': [
      {
        pattern: /(\b(?:mod)\s+)[a-z][a-z_\d]*/,
        lookbehind: true,
        alias: 'namespace',
      },
      {
        pattern: /(\b(?:crate|self|super)\s*)::\s*[a-z][a-z_\d]*\b(?:\s*::(?:\s*[a-z][a-z_\d]*\s*::)*)?/,
        lookbehind: true,
        alias: 'namespace',
        inside: {
          punctuation: /::/,
        },
      },
    ],
    keyword: {
      pattern:
        /\b(fn|impl|trait|type|mod|use|struct|if|else|for|while|loop|break|continue|return|enum|match|global|comptime|quote|unsafe|unconstrained|pub|crate|&mut|mut|self|in|as|let)\b/,
      alias: 'keyword',
    },

    builtin: {
      pattern: /\b(?:u8|u16|u32|u64|u128|i8|i16|i32|i64|i128|bool|Field|str<\d+>)\b/,
    },

    function: /\b[a-z_]\w*(?=\s*(?:::\s*<|\())/,
    macro: {
      pattern: /\b\w+!/,
      alias: 'property',
    },
    constant: /\b[A-Z_][A-Z_\d]+\b/,
    'class-name': /\b[A-Z]\w*\b/,

    namespace: {
      pattern: /(?:\b[a-z][a-z_\d]*\s*::\s*)*\b[a-z][a-z_\d]*\s*::(?!\s*<)/,
      inside: {
        punctuation: /::/,
      },
    },

    number:
      /\b(?:0x[\dA-Fa-f](?:_?[\dA-Fa-f])*|0o[0-7](?:_?[0-7])*|0b[01](?:_?[01])*|(?:(?:\d(?:_?\d)*)?\.)?\d(?:_?\d)*(?:[Ee][+-]?\d+)?)(?:_?(?:f32|f64|[iu](?:8|16|32|64|size)?))?\b/,
    boolean: /\b(?:false|true)\b/,
    punctuation: /->|\.\.=|\.{1,3}|::|[{}[\];(),:]/,
    operator: /[-+*\/%!^]=?|=[=>]?|&[&=]?|\|[|=]?|<<?=?|>>?=?|[@?]/,
  };

  Prism.languages.noir['closure-params'].inside.rest = Prism.languages.noir;
  Prism.languages.noir['attribute'].inside['string'] = Prism.languages.noir['string'];
})(Prism);
