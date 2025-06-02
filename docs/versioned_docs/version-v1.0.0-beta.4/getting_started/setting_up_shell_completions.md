---
title: Setting up shell completions
tags: []
sidebar_position: 3
---

The `nargo` binary provides a command to generate shell completions:

```bash
nargo generate-completion-script [shell]
```

where `shell` must be one of `bash`, `elvish`, `fish`, `powershell`, and `zsh`.

Below we explain how to install them in some popular shells.

## Installing Zsh Completions

If you have `oh-my-zsh` installed, you might already have a directory of automatically loading completion scripts — `.oh-my-zsh/completions`.
If not, first create it:

```bash
mkdir -p ~/.oh-my-zsh/completions`
```

Then copy the completion script to that directory:

```bash
nargo generate-completion-script zsh > ~/.oh-my-zsh/completions/_nargo
```

Without `oh-my-zsh`, you’ll need to add a path for completion scripts to your function path, and turn on completion script auto-loading. 
First, add these lines to `~/.zshrc`:

```bash
fpath=(~/.zsh/completions $fpath)
autoload -U compinit
compinit
```

Next, create a directory at `~/.zsh/completions`:

```bash
mkdir -p ~/.zsh/completions
```

Then copy the completion script to that directory:

```bash
nargo generate-completion-script zsh > ~/.zsh/completions/_nargo
```

## Installing Bash Completions

If you have [bash-completion](https://github.com/scop/bash-completion) installed, you can just copy the completion script to the `/usr/local/etc/bash_completion.d` directory:

```bash
nargo generate-completion-script bash > /usr/local/etc/bash_completion.d/nargo
```

Without `bash-completion`, you’ll need to source the completion script directly. 
First create a directory such as `~/.bash_completions/`:

```bash
mkdir ~/.bash_completions/
```

Copy the completion script to that directory:

```bash
nargo generate-completion-script bash > ~/.bash_completions/nargo.bash
```

Then add the following line to `~/.bash_profile` or `~/.bashrc`:


```bash
source ~/.bash_completions/nargo.bash
```

## Installing Fish Completions

Copy the completion script to any path listed in the environment variable `$fish_completion_path`. For example, a typical location is `~/.config/fish/completions/nargo.fish`:

```bash
nargo generate-completion-script fish > ~/.config/fish/completions/nargo.fish
```
