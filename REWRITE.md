# devconf -- Rewrite Specification

A Rust CLI tool with interactive TUI for managing development environments across Windows machines.

## Why rewrite?

Current state: PowerShell scripts that work but are sequential, non-interactive, partially idempotent, and only automate the home-laptop profile. The rewrite turns this into a proper tool with a nice UX, parallel execution, and full idempotency.

---

## 1. Technology

**Language:** Rust

Rust gives us type safety, proper error handling via `Result<T,E>`, enums for modeling states cleanly, and a single static `.exe` with zero runtime dependencies.

**Crate stack:**

| Crate                            | Role                                                                 |
| -------------------------------- | -------------------------------------------------------------------- |
| `ratatui` + `crossterm`          | TUI framework (crossterm has first-class Windows support via WinAPI) |
| `clap` (derive)                  | CLI argument parsing, subcommands, shell completions                 |
| `tokio`                          | Async runtime for parallel operations                                |
| `serde` + `serde_yaml`           | Declarative config parsing                                           |
| `color-eyre`                     | Error reporting with context                                         |
| `tracing` + `tracing-subscriber` | Structured logging to file (TUI owns stdout)                         |
| `dirs`                           | Platform-specific directories (`home`, `config`, `data`)             |

**Build & distribute:**

- `cargo build --release --target x86_64-pc-windows-msvc` on a `windows-latest` GitHub Actions runner
- Upload `.exe` to GitHub Releases on tag push
- PowerShell one-liner to install: `irm https://raw.githubusercontent.com/.../install.ps1 | iex`

---

## 2. Configuration Format

YAML, one file per profile, with a shared root config. This matches the existing repo structure (`common/`, `home-laptop/`, `work-laptop/`, `work-vm/`).

### Root config

```yaml
# devconf.yaml
schema: 1

vars:
  bin_dir: "{{home}}/.config/bin"
  language: fr-FR
```

The root config is minimal: schema version and shared variables. No defaults -- every package explicitly declares its source.

### Profile files

```yaml
# profiles/common.yaml
profile: common

packages:
  - winget: Microsoft.WindowsTerminal
  - winget: Microsoft.PowerShell
  - winget: Starship.Starship
  - winget: GitHub.Copilot
  - winget: DEVCOM.JetBrainsMonoNerdFont
  - winget: sharkdp.bat
  - winget: sharkdp.fd
  - winget: junegunn.fzf
  - winget: BurntSushi.ripgrep.MSVC
  - winget: eza-community.eza
  - winget: Microsoft.VisualStudioCode
  - winget: ZedIndustries.Zed
  - winget: Helix.Helix
  - winget: mpv.net
  - winget: WinDirStat.WinDirStat

  # Non-winget sources use the expanded form
  - github:
      repo: brequet/flatten
      asset: flatten-windows-x86_64.exe
      as: flatten.exe
      to: "{{bin_dir}}"

configs:
  - src: config/pwsh7/profile.ps1
    dest: "{{docs}}/PowerShell/Microsoft.PowerShell_profile.ps1"
    method: symlink

  - src: config/starship.toml
    dest: "{{home}}/.config/starship.toml"
    method: symlink

  - src: config/zed/settings.json
    dest: "{{appdata}}/Zed/settings.json"
    method: copy
    mkdir: true

  - src: config/.prettierrc
    dest: "{{home}}/.prettierrc"
    method: copy

  - src: config/resources/
    dest: "{{home}}/.config/resources/"
    method: copy

env:
  ZED_ALLOW_EMULATED_GPU: "1"

path:
  - "{{bin_dir}}"
  - "{{localappdata}}/Programs/mpv.net"

actions:
  - name: Set system language
    shell: Set-WinUserLanguageList -LanguageList {{language}} -Force
    admin: true
```

```yaml
# profiles/home-laptop.yaml
profile: home-laptop
extends: common

packages:
  - winget: Valve.Steam
  - scoop: extras/obs-studio

  # Subtract from parent
  - remove: WinDirStat.WinDirStat

configs:
  - src: home-laptop/pwsh7/aliases.ps1
    dest: "{{docs}}/PowerShell/home-aliases.ps1"
    method: symlink

actions:
  - name: Source home aliases in profile
    shell: |
      $target = "{{docs}}/PowerShell/home-aliases.ps1"
      $profile = "{{docs}}/PowerShell/Microsoft.PowerShell_profile.ps1"
      $line = '. "' + $target + '"'
      if ((Get-Content $profile -Raw) -notmatch [regex]::Escape($line)) {
        Add-Content $profile "`n$line"
      }
```

```yaml
# profiles/work-laptop.yaml
profile: work-laptop
extends: common

packages:
  - winget: Microsoft.AzureCLI

env:
  ZED_ALLOW_EMULATED_GPU: "1"

actions:
  - name: Map network drive
    shell: |
      $driveLetter = "S"
      $networkPath = "\\FRL205143\shared"
      if (-not (Test-Path "${driveLetter}:")) {
        $securePassword = ConvertTo-SecureString -String $env:WIN_PASSWORD -AsPlainText -Force
        $credential = New-Object PSCredential($env:WIN_USER, $securePassword)
        New-PSDrive -Name $driveLetter -PSProvider FileSystem -Root $networkPath -Credential $credential -Persist
      }
    admin: true
```

```yaml
# profiles/work-vm.yaml
profile: work-vm
extends: common

env:
  MAVEN_OPTS: "-Dfile.encoding=UTF-8 -Dconsole.encoding=UTF-8"

packages:
  - remove: mpv.net
  - remove: WinDirStat.WinDirStat
```

### Package format

Every package explicitly declares its source. The format is `source: id`:

```yaml
packages:
  # winget package
  - winget: Microsoft.WindowsTerminal

  # scoop package
  - scoop: extras/obs-studio

  # GitHub release binary (expanded form for extra options)
  - github:
      repo: owner/name
      asset: pattern-for-asset.exe
      as: local-name.exe         # optional: rename the downloaded file
      to: "{{bin_dir}}"          # optional: target directory

  # Chocolatey package
  - choco: some-package

  # Remove a package inherited from parent profile
  - remove: package-id
```

No implicit source, no defaults. You always see where a package comes from by looking at the line.

### Per-package hooks

Any package can have optional `before` and `after` fields to run shell commands around the install:

```yaml
packages:
  - winget: Some.Package
    after: refreshenv

  - winget: Another.Package
    before: |
      Write-Host "Preparing environment..."
    after: |
      refreshenv
      some-post-setup-command --init
```

The engine runs: `before` (if any) -> install/upgrade -> `after` (if any), sequentially for that package. Hooks are PowerShell commands, same as `actions:`. If `before` fails, the install is skipped and marked as failed.

Global post-install logic belongs in `actions:`, which runs after all packages and configs are processed.

### Config format

Every config entry uses `src`, `dest`, and `method`. The method is mandatory -- always `symlink` or `copy`:

```yaml
configs:
  - src: config/starship.toml
    dest: "{{home}}/.config/starship.toml"
    method: symlink

  - src: config/zed/settings.json
    dest: "{{appdata}}/Zed/settings.json"
    method: copy
    mkdir: true          # optional: create parent directories
```

No shorthand, no implicit defaults. Every config entry is explicit about what it does.

### Template variables

| Variable           | Resolves to                               |
| ------------------ | ----------------------------------------- |
| `{{home}}`         | `$HOME` / `$USERPROFILE`                  |
| `{{appdata}}`      | `$APPDATA`                                |
| `{{localappdata}}` | `$LOCALAPPDATA`                           |
| `{{docs}}`         | `$HOME\Documents`                         |
| `{{repo_root}}`    | Root of the dev-conf repo                 |
| `{{profile_root}}` | Directory of the current profile          |
| Custom vars        | Defined in `devconf.yaml` `vars:` section |

### Design rationale

- **Explicit source on every package.** `- winget: X` is barely more typing than `- X` and removes all ambiguity. You never have to look at a defaults block to know where a package comes from.
- **No groups.** A flat list is simpler to read, edit, and diff. If you want to visually organize packages, use YAML comments. The TUI can sort/categorize by source automatically.
- **Mandatory `method` on configs.** No implicit behavior. Every config entry says exactly what it does -- `symlink` or `copy`. Trivial to deserialize, trivial to understand.
- **Per-package hooks.** `before:` and `after:` keep hook logic scoped to the package it relates to. No spooky action at a distance.
- **`remove:` in child profiles.** Solves the real problem: "work-vm needs common minus media apps."
- **One file per profile.** Clean git diffs, each machine's config is self-contained, no merge conflicts.
- **`extends:` is single-parent.** Simple. If shared subsets are needed later, add `includes: [file.yaml]`.

---

## 3. Architecture

```
src/
  main.rs                 # Entry point, #[tokio::main], clap dispatch
  cli.rs                  # Clap derive structs (subcommands, flags)

  config/
    mod.rs                # Profile loading, merging, validation
    schema.rs             # Serde structs: Profile, Package, Config, Action, etc.
    vars.rs               # Template variable resolution

  engine/
    mod.rs                # Orchestrator: resolve profile -> check -> install
    check.rs              # Parallel status checking across all providers
    install.rs            # Parallel installation dispatch
    config_deploy.rs      # Symlink/copy with idempotency checks

  providers/
    mod.rs                # Provider trait definition
    winget.rs             # winget list / install / upgrade
    github.rs             # GitHub releases API
    scoop.rs              # scoop info / install
    choco.rs              # choco list / install

  tui/
    mod.rs                # Terminal setup, main event loop
    screens/
      dashboard.rs        # Overview: all packages + status table
      selector.rs         # Checkbox list for interactive install
      installer.rs        # Parallel progress view during installation
      profile_picker.rs   # Choose a profile
      doctor.rs           # Health check results
      summary.rs          # Post-install report

  system/
    mod.rs
    env.rs                # Read/set environment variables
    path.rs               # PATH management (add, check)
    shell.rs              # Execute PowerShell commands/scripts
```

### Provider trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self, package: &Package) -> Result<PackageStatus>;
    async fn install(&self, package: &Package) -> Result<()>;
    async fn upgrade(&self, package: &Package) -> Result<()>;
}

pub enum PackageStatus {
    NotInstalled,
    Installed { version: String },
    Outdated { current: String, available: String },
    Unknown,
}
```

Each provider wraps shell commands (`winget`, `choco`, `scoop`) or HTTP calls (GitHub API). The engine dispatches to the right provider based on the package's declared source.

### Parallel execution model

```
Main thread (TUI render loop)
  |
  |  tokio::select! {
  |    event = crossterm_events.next() => handle_input(),
  |    update = task_rx.recv()         => update_state(),
  |    _ = tick_interval.tick()        => render(),
  |  }
  |
  +--- mpsc channel <-- tokio::spawn(check bat)
  |                 <-- tokio::spawn(check fd)
  |                 <-- tokio::spawn(check fzf)
  |                 <-- tokio::spawn(install zed)
  |                 <-- tokio::spawn(install ripgrep)
  |                 <-- ...
```

Each package check/install runs in its own tokio task. Tasks send status updates (`Checking`, `Installing`, `Upgrading`, `Done`, `Failed`) back to the TUI via an mpsc channel. The TUI renders a live table.

---

## 4. Idempotency Rules

Every operation checks current state before acting:

| Resource       | Check                                                                                                            | Action                                                                                          |
| -------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Winget package | `winget list --id X --exact`                                                                                     | Missing -> install. Outdated -> upgrade. Current -> skip.                                       |
| GitHub binary  | File exists at target path. Compare local version tag (stored in state file) with GitHub API `/releases/latest`. | Missing -> download. Outdated -> re-download. Current -> skip.                                  |
| Scoop package  | `scoop info X` exit code                                                                                         | Same pattern as winget.                                                                         |
| Choco package  | `choco list X --exact --local-only`                                                                              | Same pattern.                                                                                   |
| Symlink        | `is_symlink(dest) && read_link(dest) == expected_src`                                                            | Correct -> skip. Wrong target -> remove + re-create. Not a symlink -> backup original + create. |
| File copy      | Compare SHA-256 hash of src vs dest                                                                              | Same hash -> skip. Different -> backup dest + copy.                                             |
| Env variable   | `[Environment]::GetEnvironmentVariable(name, "User")`                                                            | Same value -> skip. Different -> set.                                                           |
| PATH entry     | Check if dir is present in current user PATH                                                                     | Present -> skip. Missing -> append.                                                             |

**State file:** `~/.config/devconf/state.json` tracks GitHub binary versions and last-run timestamps. Minimal -- only what can't be determined from the system itself.

---

## 5. CLI Commands

```
devconf                          # Default: interactive TUI
devconf install                  # Interactive: choose profile, select apps, install
devconf install --all            # Non-interactive: install everything for active profile
devconf install --profile work   # Use a specific profile
devconf status                   # Show what's installed, outdated, missing
devconf sync                     # Deploy config files, env vars, PATH only (no apps)
devconf upgrade                  # Upgrade all outdated packages
devconf doctor                   # Verify system health: configs correct, apps present, env set
devconf profile list             # List available profiles
devconf profile set <name>       # Set the active profile (saved in state file)
devconf export                   # Scan current machine -> generate a profile YAML
```

**Flags available on most commands:**

- `--dry-run` -- show what would happen without doing anything
- `--no-tui` -- plain text output (for piping, CI, scripts)
- `--verbose` / `-v` -- detailed output
- `--parallel N` -- max concurrent operations (default: 4)

---

## 6. TUI Screens

### Dashboard (default `devconf status` view)

```
╭─ devconf ─ home-laptop ───────────────────────────────────╮
│                                                            │
│  winget              bat         ✓ installed   0.24.0      │
│                      fd          ✓ installed   10.2.0      │
│                      fzf         ↑ outdated    0.55 → 0.56 │
│                      ripgrep     ✓ installed   14.1.0      │
│                      eza         ✓ installed   0.20.0      │
│                      VS Code     ✓ installed   1.96        │
│                      Zed         ✗ missing     —           │
│                      Helix       ✓ installed   24.7        │
│                      PowerShell  ✓ installed   7.5.0       │
│                      Starship    ✓ installed   1.22        │
│                      Terminal    ✓ installed   1.21        │
│                                                            │
│  github              flatten     ✓ installed   v1.2.3      │
│                                                            │
│  scoop               obs-studio  ✓ installed   30.2        │
│                                                            │
│  [i]nstall  [u]pgrade  [s]ync  [d]octor  [q]uit           │
╰────────────────────────────────────────────────────────────╯
```

The TUI groups packages by source automatically -- no need for manual groups in the config.

### Installer (parallel progress)

```
╭─ Installing ─ home-laptop ────────────────────────────────╮
│                                                            │
│  ✓  bat            installed                     0.3s      │
│  ✓  fd             installed                     0.4s      │
│  ↻  fzf            upgrading 0.55 → 0.56...     ━━━━━━━╸  │
│  ✓  ripgrep        installed                     0.2s      │
│  ↻  Zed            installing...                 ━━━━╸     │
│  ·  Helix          waiting...                              │
│  ·  VS Code        waiting...                              │
│                                                            │
│  5/15 complete  ·  2 in progress  ·  0 failed              │
╰────────────────────────────────────────────────────────────╯
```

### Selector (interactive checkbox)

```
╭─ Select packages to install ──────────────────────────────╮
│                                                            │
│  winget                                                    │
│    [x] bat                         ✓ installed             │
│    [x] fd                          ✓ installed             │
│    [x] fzf                         ↑ upgrade available     │
│    [x] ripgrep                     ✓ installed             │
│    [ ] eza                         ✗ not installed         │
│    [x] VS Code                     ✓ installed             │
│    [ ] Zed                         ✗ not installed         │
│    [x] Helix                       ✓ installed             │
│                                                            │
│  github                                                    │
│    [x] flatten                     ✓ installed             │
│                                                            │
│  Space: toggle  ·  a: all  ·  n: none  ·  Enter: confirm  │
╰────────────────────────────────────────────────────────────╯
```

---

## 7. Repo Structure After Rewrite

```
dev-conf/
  devconf.yaml                    # Root config (vars only)
  profiles/
    common.yaml                   # Shared base profile
    home-laptop.yaml
    work-laptop.yaml
    work-vm.yaml
  config/                         # Dotfiles and configs (unchanged)
    pwsh7/profile.ps1
    starship.toml
    zed/settings.json
    resources/sfx/
    .prettierrc
  home-laptop/                    # Machine-specific dotfiles
    pwsh7/aliases.ps1
  src/                            # Rust source code
    main.rs
    cli.rs
    config/
    engine/
    providers/
    tui/
    system/
  Cargo.toml
  Cargo.lock
  install.ps1                     # One-liner bootstrap for fresh machines
  .github/
    workflows/
      release.yml                 # Build + release on tag push
  .gitignore
```

---

## 8. Implementation Phases

### Phase 1 -- Foundation (get it running)

- [ ] Project skeleton: `Cargo.toml`, module structure, clap CLI
- [ ] Config schema: serde structs, YAML parsing, profile merging (`extends` + `remove`)
- [ ] Template variable resolution (`{{home}}`, `{{appdata}}`, etc.)
- [ ] Provider trait + winget provider (check / install / upgrade)
- [ ] GitHub releases provider
- [ ] Sequential engine (check all, install all) with plain text output
- [ ] `devconf install --all --profile <name>` working end-to-end

### Phase 2 -- Parallel & TUI

- [ ] Tokio-based parallel check and install engine
- [ ] ratatui terminal setup (alternate screen, raw mode, panic hooks)
- [ ] Dashboard screen (status table, grouped by source)
- [ ] Installer screen (parallel progress)
- [ ] Selector screen (checkbox list)
- [ ] Profile picker screen

### Phase 3 -- Config & System

- [ ] Config file deployment (symlink / copy, idempotent)
- [ ] Environment variable management
- [ ] PATH management
- [ ] PowerShell script execution (`actions:` blocks)
- [ ] `devconf sync` command
- [ ] `devconf doctor` command

### Phase 4 -- Extra Providers & Polish

- [ ] Scoop provider
- [ ] Chocolatey provider
- [ ] `--dry-run` flag
- [ ] `--no-tui` flag (plain text fallback)
- [ ] State file (`~/.config/devconf/state.json`)
- [ ] Logging to file via tracing
- [ ] Error recovery / retry on failed installs

### Phase 5 -- Distribution & Extras

- [ ] GitHub Actions release workflow
- [ ] `install.ps1` bootstrap script
- [ ] `devconf export` (scan machine -> generate YAML)
- [ ] Shell completions (PowerShell)
- [ ] Sound notifications on completion (use existing sfx files)
- [ ] Drift detection idea: scheduled `devconf doctor` via Task Scheduler

---

## 9. Open Questions

Things to decide as you build:

1. **Admin elevation strategy.** Some installs need admin. Options:
   - Require running as admin always (current approach)
   - Self-elevate via `Start-Process -Verb RunAs` when needed
   - Separate admin and non-admin operations, prompt when elevation is needed

2. **Auto-install package managers.** If the profile specifies `scoop:` but scoop isn't installed, should devconf install scoop automatically? Probably yes, with a confirmation prompt.

3. **Config conflicts.** If a child profile `remove:`s a package but keeps a config depending on it (e.g., remove Zed but keep Zed settings), should devconf warn? Nice-to-have but not critical.

4. **Winget concurrency limits.** Winget may not handle multiple concurrent installs well (shared locks on package databases). May need to limit winget to sequential or small batches while running other providers in parallel. Test this early.

5. **GitHub binary versioning.** GitHub releases don't have a "list installed version" concept. The state file needs to track `{ "brequet/flatten": "v1.2.3" }` so upgrades can compare against the latest release tag.

6. **TOML vs YAML.** This spec uses YAML because it's more readable for large config lists and supports multiline strings cleanly. TOML's `[[array]]` syntax gets verbose with 15+ packages. But TOML has stronger typing -- revisit if YAML parsing causes issues.

---

## 10. Ideas Backlog

Things that aren't in the main plan but could be fun later:

- **Windows Terminal settings deployment** -- configure themes, fonts, default profile
- **VS Code extensions sync** -- export/import extension list
- **Registry tweaks** -- show file extensions, disable Bing search, enable developer mode
- **AppX removal** -- uninstall pre-installed bloatware
- **Git config** -- set user name, email, default editor, SSH key generation
- **Encrypted secrets** -- store sensitive env vars (tokens, passwords) encrypted in the repo, decrypt at deploy time
- **Catppuccin theme for the TUI** -- match the existing starship theme
- **`devconf diff`** -- show what's different between current machine state and the profile
- **Self-update** -- `devconf update` downloads the latest release and replaces itself
- **Plugin system** -- custom providers as external executables (like git's `git-<command>` pattern)
