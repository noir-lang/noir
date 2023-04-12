# Docs

To generate docs. Go to the root of `yarn-project` and run:

```bash
yarn docs
```

This will generate the html, and start a server to expose it on port `8080`, if run from external server (the mainframe) you can add `LocalForward 8080 <IP>:8080` to your ssh-config, and access it from your browser.

## Adding a new package

To include a new package in the set that we generate documentation for add it to the `entrypoints` list in the `yarn-project/typedoc.json`, then in the `package.json` for the new package add typedoc similar to example below:

```json
  "typedoc": {
    "entryPoint": "./src/index.ts",
    "displayName": "Aztec cli",
    "tsconfig": "./tsconfig.json"
  },
```
