# zgen
if [ ! -d "$HOME/.zgen" ]; then
  git clone https://github.com/tarjoilija/zgen.git "${HOME}/.zgen"
fi

source "${HOME}/.zgen/zgen.zsh"

if ! zgen saved; then
  # specify plugins here
  zgen oh-my-zsh
  zgen oh-my-zsh plugins/git
  zgen oh-my-zsh plugins/vi-mode
  zgen oh-my-zsh plugins/fzf
  zgen load miekg/lean

  # generate the init script from plugins above
  zgen save
fi

function zle-keymap-select zle-line-init
{
    # change cursor shape in iTerm2
    case $KEYMAP in
        vicmd)      echo -ne '\e[1 q';;
        viins|main) echo -ne '\e[5 q';;
    esac

    zle reset-prompt
    zle -R
}

function zle-line-finish
{
    print -n -- "\E]50;CursorShape=0\C-G"  # block cursor
}

zle -N zle-line-init
zle -N zle-line-finish
zle -N zle-keymap-select

setopt no_share_history
setopt rm_star_silent
setopt auto_pushd
setopt +o nomatch
set +o histexpand

bindkey    "^[[3~"          delete-char
bindkey    "^[3;5~"         delete-char
bindkey    "^[[A"           history-search-backward
bindkey    "^[[B"           history-search-forward

# Prevent mad background colors on permissive permissions.
export LS_COLORS="di=34:ln=36:so=35:pi=33:ex=32:bd=1;33:cd=1;33:su=31:sg=32:tw=34:ow=34:st=37"

# Colorize completions using default `ls` colors.
zstyle ':completion:*' list-colors ''
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"

export MAKEFLAGS=-j$(nproc)

alias dr="docker run -ti --rm"
alias drs="docker run -ti --rm --entrypoint /bin/sh"
alias vim=nvim

# Graphite aliases
alias gtl="gt log"
alias gtd="gt down"
alias gtu="gt up"
alias gts="gt sync"
alias gto="gt checkout"

# Fuzzy git rooted dir change on ctrl-f.
gitcd() {
  git_root=$(git rev-parse --show-toplevel 2> /dev/null)
  if [[ $? -eq 0 ]]; then
    local selected_dir=$(cd "$git_root" && find * -type d -not -path '*node_modules*' -not -path '.git*' | fzf)
    if [[ -n "$selected_dir" ]]; then
      # Instead of changing directory, prepare a cd command
      BUFFER="cd \"$git_root/$selected_dir\""
      zle accept-line
    fi
  fi
}
zle -N gitcd_widget gitcd
bindkey '^F' gitcd_widget

# Graphite autocomplete.
#compdef gt
###-begin-gt-completions-###
#
# yargs command completion script
#
# Installation: gt completion >> ~/.zshrc
#    or gt completion >> ~/.zprofile on OSX.
#
_gt_yargs_completions()
{
  local reply
  local si=$IFS
  IFS=$'
' reply=($(COMP_CWORD="$((CURRENT-1))" COMP_LINE="$BUFFER" COMP_POINT="$CURSOR" gt --get-yargs-completions "${words[@]}"))
  IFS=$si
  _describe 'values' reply
}
compdef _gt_yargs_completions gt
###-end-gt-completions-###
