#!/usr/bin/env bash

apt update
apt install gh
gh codespace ports visibility 8080:public -c $CODESPACE_NAME

npx aztec-app sandbox start

r=$(tput sgr0)       # Reset color
bold=$(tput bold)    # Bold text
g=$(tput setaf 46)   # Light Green
b=$(tput setaf 21)   # Bright Blue
p=$(tput setaf 13)   # Magenta
y=$(tput setaf 226)  # Bright Yellow
c=$(tput setaf 51)   # Cyan
o=$(tput setaf 208)  # Orange

# Function to print colored text
print_colored() {
  case $2 in
    green)
      color=$g
      ;;
    blue)
      color=$b
      ;;
    magenta)
      color=$p
      ;;
    yellow)
      color=$y
      ;;
    cyan)
      color=$c
      ;;
    orange)
      color=$o
      ;;
    *)
      color=$r
      ;;
  esac
  echo "${color}$1${r}"
}

echo
echo "${bold}${c} █████╗ ███████╗████████╗███████╗ ██████╗${r}"
echo "${bold}${o}██╔══██╗╚══███╔╝╚══██╔══╝██╔════╝██╔════╝${r}"
echo "${bold}${g}███████║  ███╔╝    ██║   █████╗  ██║${r}"
echo "${bold}${b}██╔══██║ ███╔╝     ██║   ██╔══╝  ██║${r}"
echo "${bold}${p}██║  ██║███████╗   ██║   ███████╗╚██████╗${r}"
echo "${bold}${y}╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝${r}"
echo
print_colored "${bold}Your codespace is ready with your chosen box! 🎉" "cyan"
print_colored "You can now yarn dev or any other package.json script." "cyan"
echo
print_colored "Manage the running development network by running:" "magenta"
print_colored "sandbox [start, stop, logs, etc]" "green"
print_colored "example: \"sandbox logs\""
echo
print_colored "You can also connect your local environment to it." "magenta"
print_colored "To do so, prepend your commands with this codespace's sandbox URL:" "magenta"
print_colored "${PXE_URL}" "green" 
print_colored "example: PXE_URL=\"${PXE_URL}\" yarn dev"
echo
print_colored "${bold}Enjoy your sandbox! 🏖️" "orange"
