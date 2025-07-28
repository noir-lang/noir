# Docusaurus Plugin: Experimental Disclaimer Injector

A custom Docusaurus plugin for injecting the [experimental disclaimer](../../src/components/Notes/_experimental.mdx) into specified documentation directories, e.g. automatically generated reference documentations.

## Usage

Add the plugin to your `docusaurus.config.js`, and specify targets to inject:

```javascript
plugins: [
  [
    './plugins/docusaurus-plugin-experimental-disclaimer',
    {
    targets: [
      'processed-docs/reference/NoirJS/noir_wasm',
      'processed-docs/reference/some_other_experimental_tooling'
    ]
  }
  ]
]
```

## How It Works

1. The plugin runs during Docusaurus's `loadContent` lifecycle
2. It searches for all `.mdx` files in the specified target directories
3. For each file, it checks if the disclaimer already exists using the skip check pattern
4. If not present, it injects the disclaimer after the first heading (`# title`) or at the beginning of the file
5. The modified files are then processed by Docusaurus during the normal build process

## Custom Disclaimers

The plugin also experimentally support for custom disclaimers.

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `targets` | `string[]` | `[]` | Array of directories to process (relative to site directory) |
| `disclaimer` | `string` | Default experimental disclaimer | Custom disclaimer content to inject |
| `skipCheckPattern` | `string` | `"import Experimental from '@site/src/components/Notes/_experimental.mdx';"` | Pattern to check if disclaimer already exists |

### Usage

#### Imported Custom Disclaimers

Add the plugin to your `docusaurus.config.js`, and specify targets and disclaimer to inject, and disclaimer pattern check:

```javascript
plugins: [
  [
    './plugins/docusaurus-plugin-experimental-disclaimer',
    {
      targets: ['processed-docs/reference/NoirJS/noir_wasm'],
      // Optional: custom disclaimer content
      disclaimer: `import MyDisclaimer from '@site/src/components/MyDisclaimer.mdx';

<MyDisclaimer />

`,
      // Optional: custom skip check pattern
      skipCheckPattern: 'import MyDisclaimer'
    }
  ]
]
```

#### Inline Custom Disclaimers

Instead of importing, you can also define the custom disclaimer in-line.

```javascript
[
  './plugins/docusaurus-plugin-experimental-disclaimer',
  {
    targets: ['processed-docs/reference/beta-api'],
    disclaimer: `:::warning Beta Feature
This API is in beta and may change in future releases.
:::

`,
    skipCheckPattern: ':::warning Beta Feature'
  }
]
```



## Example: Multiple Targets

```javascript
[
  './plugins/docusaurus-plugin-experimental-disclaimer',

]
```
