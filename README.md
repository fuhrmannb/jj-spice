# jj-spice

Forge integration for [jj (Jujutsu)](https://github.com/jj-vcs/jj). Manage
stacked change requests from the command line.

## Shell completion

jj-spice supports two completion methods: **dynamic** (recommended) and
**static**.

Dynamic completion calls back into `jj-spice` at TAB-time, so completions
stay in sync with the installed version and can complete config keys and
values. Static completion generates a one-time script that only knows about
subcommands and flags.

### Dynamic completion (recommended)

Add one of the following to your shell startup file:

**Bash** (`~/.bashrc`):

```bash
source <(COMPLETE=bash jj-spice)
```

**Zsh** (`~/.zshrc`):

```zsh
source <(COMPLETE=zsh jj-spice)
```

**Fish** (`~/.config/fish/config.fish`):

```fish
COMPLETE=fish jj-spice | source
```

**Elvish**:

```elvish
eval (COMPLETE=elvish jj-spice | slurp)
```

**PowerShell**:

```powershell
COMPLETE=powershell jj-spice | Out-String | Invoke-Expression
```

### Static completion

Generate a static script with `jj-spice util completion <SHELL>`. This is
useful when dynamic completion is not available (e.g. Nushell) or when you
prefer a pre-generated script.

**Bash**:

```bash
jj-spice util completion bash > ~/.local/share/bash-completion/completions/jj-spice
```

**Zsh** (write to a directory in your `$fpath`):

```zsh
jj-spice util completion zsh > ~/.zfunc/_jj-spice
```

**Fish**:

```fish
jj-spice util completion fish > ~/.config/fish/completions/jj-spice.fish
```

**Nushell** (static only — dynamic completion is not supported):

```nu
jj-spice util completion nushell | save -f "completions-jj-spice.nu"
use "completions-jj-spice.nu" *
```

**PowerShell**:

```powershell
jj-spice util completion power-shell | Out-String | Invoke-Expression
```

**Elvish**:

```elvish
eval (jj-spice util completion elvish | slurp)
```