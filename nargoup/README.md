# `nargoup`

Update or revert to a specific Nargo branch with ease.

## Installing

```sh
curl -L https://raw.githubusercontent.com/noir-lang/noir/master/nargoup/install | bash
```

## Usage

To install the **nightly** version:

```sh
nargoup
```

To install a specific **version** (in this case the `nightly` version):

```sh
nargoup --version nightly
```

To install a specific **branch** (in this case the `release/0.1.0` branch's latest commit):

```sh
nargoup --branch release/0.1.0
```

To install a **fork's main branch** (in this case `tomafrench/noir`'s main branch):

```sh
nargoup --repo tomafrench/noir
```

To install a **specific branch in a fork** (in this case the `patch-10` branch's latest commit in `tomafrench/noir`):

```sh
nargoup --repo tomafrench/noir --branch patch-10
```

To install from a **specific Pull Request**:

```sh
nargoup --pr 367
```

To install from a **specific commit**:

```sh
nargoup -C 20048e7
```

To install a local directory or repository (e.g. one located at `~/git/noir`, assuming you're in the home directory)

##### Note: --branch, --repo, and --version flags are ignored during local installations.

```sh
nargoup --path ./git/noir
```

---

**Tip**: All flags have a single character shorthand equivalent! You can use `-v` instead of `--version`, etc.

---
