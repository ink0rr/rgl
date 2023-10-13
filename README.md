# rgl

Not Regolith.

### Install

Shell (Mac, Linux):

```sh
curl -fsSL https://raw.githubusercontent.com/ink0rr/rgl/main/scripts/install.sh | sh
```

PowerShell (Windows):

```powershell
irm https://raw.githubusercontent.com/ink0rr/rgl/main/scripts/install.ps1 | iex
```

## Compatibility

- The Shell installer can be used on Windows with [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/about), [MSYS](https://www.msys2.org) or equivalent set of tools.

## Known Issues

### unzip is required

The program [`unzip`](https://linux.die.net/man/1/unzip) is a requirement for the Shell installer.

**When does this issue occur?**

During the `install.sh` process, `unzip` is used to extract the zip archive.

**How can this issue be fixed?**

You can install unzip via `brew install unzip` on MacOS or `apt-get install unzip -y` on Linux.
