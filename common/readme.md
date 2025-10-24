# Windows setup

## Apps

- PowerShell 7: `winget install -e Microsoft.PowerShell --source winget`
- bat: `winget install -e sharkdp.bat`
- fd: `winget install -e sharkdp.fd`
- fzf: `winget install -e junegunn.fzf`
- zed: `winget install -e ZedIndustries.Zed`
- ripgrep: `winget install --id=BurntSushi.ripgrep.MSVC -e`
- helix: `winget install --id=Helix.Helix -e`
- Jetbrain Mono: ` winget install --id=DEVCOM.JetBrainsMonoNerdFont -e`
- lstr: todo

Install with command --id and -e for exact match.

Only install apps not already installed -> winget handles it.

Install app script MUST be run as admin ! Add a check at the beginning to enforce this.

Also make it so the install script can be run on itself: upgrade thingy if possible, else if NTD then pass to the next step.

## Configuration

- Terminal:
  - Oh-My-Posh -> https://ohmyposh.dev/docs/installation/prompt
  - Jetbrain Mono -> find a way to script pwsh7 to use it
- Zed:
  - Inject the conf in config.json: copy the one here to location `"$env:APPDATA\Zed\settings.json"` (folder may not exists)
