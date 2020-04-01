export TERM=vt100
alias ls='ls --color=auto'
export PS1="\[\033[38;5;10m\]\W\[$(tput sgr0)\] \[$(tput sgr0)\]\[\033[38;5;14m\]\\$\[$(tput sgr0)\] \[$(tput sgr0)\]"
export PATH="$PATH:/.cargo/bin"
