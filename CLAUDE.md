# Noir Monorepo Development Guide

## What is Noir?

Noir is a Domain Specific Language for SNARK proving systems, designed to work with any ACIR compatible proving system. See [README.md](./README.md) for links to documentation, getting started guides, and community resources.

## Repository Map

| Directory | Description |
|-----------|-------------|
| `acvm-repo/` | ACIR Virtual Machine - IR definitions, Brillig VM, black box solvers |
| `compiler/` | Noir compiler - frontend, evaluator, driver, error handling |
| `docs/` | Documentation site content |
| `examples/` | Example Noir projects |
| `noir_stdlib/` | Noir standard library |
| `scripts/` | Build and CI scripts |
| `test_programs/` | Compiler test cases |
| `tooling/` | Developer tools - nargo CLI, LSP, debugger, formatters, profiler |

## Tools

### Just (Command Runner)

This project uses [just](https://github.com/casey/just) as a command runner (similar to `make`). See [`justfile`](./justfile) for all available recipes.

Common recipes:
- `just install-tools` - Install all development dependencies
- `just test` - Run Rust tests with nextest
- `just format` / `just clippy` - Rust formatting and linting
- `just format-noir` - Format Noir code
- `just lint` - Lint TypeScript/JavaScript code

Run `just --list` to see all available commands.

## 🚀 Essential Workflow

### Before Running Javascript Tests - ALWAYS COMPILE

```bash
yarn build  # Full compilation
# OR for specific package:
cd <package-name> && yarn build
```

### Before Committing - Quality Checklist

1. **Build**: Ensure project compiles (`yarn tsc -b`)
2. **Format/Lint**: Run on modified packages (see Format & Lint section)
3. **Test**: Run unit tests for modified files and ensure they pass

## 📦 Compilation

### Full Project

```bash
yarn build
```

### Specific Package

```bash
cd <package-name>
yarn build
```

## 🧪 Testing

**⚠️ NEVER run `yarn test` from the project root - ALWAYS cd into a specific package first!**

### Standard Tests

```bash
# WRONG: yarn test from repository root ❌
# RIGHT: Always cd into package first ✅

cd <package-name>
yarn test FILENAME                    # Run test file
yarn test FILENAME -t 'test-name'     # Run specific test
```

## 🎨 Format & Lint

### Apply Changes

```bash
# Rust code
just format
just clippy

# Noir code
just format-noir

# Typescript code
just lint
```

### Check Mode (No Changes)

Rust code can be checked for formatting changes without applying them

```bash
just format-check
```

## 📦 Dependency Management

After modifying any `package.json`:

```bash
yarn install
```

## 🔀 Git & PR Guidelines

### Branch Naming

Prefix branches with author initials:

```
ab/feature-name
jd/fix-something
```

**Setting Author Initials:**
Configure your git initials for automatic branch naming:

```bash
# Local repository only
git config user.initials "jd"

# Global (all repositories)
git config --global user.initials "jd"
```

**How Claude Determines Author Initials:**

1. First checks `git config user.initials`
2. If not set, derives from `git config user.name` (e.g., "John Doe" → "jd")
3. Uses lowercase initials for branch names
4. Ask to set the user's initials for them if unset.

### Commit Messages - Conventional Commits

Follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)

**Supported types** (from `.github/workflows/pull-request-title.yml`):

- `fix`: Bug fixes
- `feat`: New features
- `chore`: Maintenance tasks

**Format**:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Branch Strategy

- **Primary development**: `master` branch (default PR target)
- **Default PR target**: `master` (unless specified otherwise)

### Breaking Changes

When introducing breaking changes:

1. **Include in PR description**: Clearly document the breaking changes

### CI Labels

Special labels to control CI behavior:

- **`show-bench`**: Print the comparisons of CI benchmarks between the PR commit and the base branch.

  - Use when you think that the PR will result in improvements/degradation of compilation time or memory usage.

## 📚 Quick Reference

### Common Package Commands

```bash
# Compile
yarn build

# Test (MUST cd into package first!)
cd package-name
yarn test filename.test.ts
yarn test filename.test.ts -t 'specific test'
```

### Workflow Reminders

- ✅ Always compile before testing
- ✅ Format/lint modified packages before committing
- ✅ Run tests for modified code
- ✅ Use single-package commands when possible (faster)
- ❌ Never run `yarn test` from project root - always cd into package first
