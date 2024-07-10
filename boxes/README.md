# Aztec-App

Aztec-App is a set of tools to ease development on Aztec. It consists of two main components:
1. `npx` script
2. boxes (starter kits)
## npx script

NPX is a tool bundled with `npm` and other package managers. It allows you to run a binary from a cache without installing it globally.

To ease the development process, Aztec has developed this binary. To run it, install Node and run:

```bash
npx aztec-app
```

This will prompt you with some options to clone `Aztec Boxes` and install the sandbox. As the `npx` script evolves, other commands will be added or removed. You can run it with the `-h` flag to know what other commands and flags you can pass to it.

> [!NOTE]  
> As a tool that doesn't (yet) have automated testing, it versioning and release process is decoupled from `aztec`, and its deployment is entirely manual by running `yarn npm publish --access public`.

## Aztec Boxes

Aztec Boxes are the one-stop-shop for developing on Aztec. They often include a combination of:

- Fully tested contracts
- Frontend boilerplates
- General tests

Boxes include the sandbox installation script and its start command. By choosing the appropriate box, you can get started working on Aztec in a minimal amount of time.

## Contributing

Because of the CI/CD nature of the monorepo, every box is tested against every merge on master. This drastically reduces their maintenance cost. Thus, some scripting is needed to make sure the user gets a working repository after "unboxing".

Most of the logic is in the `bin.js` file, where `commander` commands stuff. The script does the following:

- Prompts the user for options and commands
- Inits some global variables such as a logger, a getter for the github repositories, the latest stable versions and tags, etc
- Prompts the user to choose the project and clone it. It then rewrites the `Nargo.toml` and `package.json` files to point to the repos instead of the local dependencies.
- Queries the local docker daemon for any existing sandbox images, prompting the user to install or update it if needed
- Asks the user if they want to run the sandbox right away


## Templates

As noted above, every box is tested at every merge to master. Any breaking changes need to happen in every box, so we try to keep the number of templates strategically low. For that reason, we ask contributors to reach directly to the [devrel team](https://github.com/orgs/AztecProtocol/teams/devrel) before adding another template.

Currently there are two "app" boxes and one "contract-only" box:

- React - A React boilerplate with a minimal UI.
- Vanilla JS and HTML - Some say if you get something working in vanilla JS and HTML, you can make it work on any framework. If you can't find the box you need, this could be a good starting point.
- Token - An example token contract on Aztec

## Support

Need any support? Reach out on [discord](https://discord.gg/DgWG2DBMyB), [discourse](https://discourse.aztec.network/) or [twitter](https://twitter.com/aztecnetwork).
