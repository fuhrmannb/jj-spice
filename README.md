# jj-spice

Forge integration for [jj (Jujutsu)](https://github.com/jj-vcs/jj). Manage
stacked change requests from the command line.

## Shell completion

Generate a completion script with `jj-spice util completion <SHELL>`.

### Bash

```bash
source <(jj-spice util completion bash)
```

To persist across sessions, write the script to the completions directory:

```bash
jj-spice util completion bash > ~/.local/share/bash-completion/completions/jj-spice
```

### Zsh

```zsh
autoload -U compinit
compinit
source <(jj-spice util completion zsh)
```

To persist, write the script to a directory in your `$fpath`:

```zsh
jj-spice util completion zsh > ~/.zfunc/_jj-spice
```

### Fish

```fish
jj-spice util completion fish | source
```

To persist:

```fish
jj-spice util completion fish > ~/.config/fish/completions/jj-spice.fish
```

### Nushell

```nu
jj-spice util completion nushell | save -f "completions-jj-spice.nu"
use "completions-jj-spice.nu" *
```

### PowerShell

```powershell
jj-spice util completion power-shell | Out-String | Invoke-Expression
```

### Elvish

```elvish
eval (jj-spice util completion elvish | slurp)
```