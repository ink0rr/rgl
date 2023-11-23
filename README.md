# rgl

Fast and minimal implementation of Regolith.

## Benchmark

Benchmark result on a project with ~2300 files, no filters, and a total size of 9.2MB

```
$ hyperfine --warmup 2 --runs 10 --setup 'rgl clean' 'regolith run build' 'rgl run build' 'rgl run build --cached'

Benchmark 1: regolith run build
  Time (mean ± σ):     511.7 ms ±  24.0 ms    [User: 43.8 ms, System: 465.5 ms]
  Range (min … max):   485.0 ms … 560.6 ms    10 runs

Benchmark 2: rgl run build
  Time (mean ± σ):     186.9 ms ±  34.3 ms    [User: 24.0 ms, System: 789.0 ms]
  Range (min … max):   134.9 ms … 257.4 ms    10 runs

Benchmark 3: rgl run build --cached
  Time (mean ± σ):      46.2 ms ±   1.1 ms    [User: 26.2 ms, System: 245.2 ms]
  Range (min … max):    43.5 ms …  47.6 ms    10 runs

Summary
  rgl run build --cached ran
    4.05 ± 0.75 times faster than rgl run build
   11.08 ± 0.59 times faster than regolith run build
```

> Testen on a MacBook Pro M1 (2020)

### Install

Shell (Mac, Linux):

```sh
curl -fsSL https://raw.githubusercontent.com/ink0rr/rgl/main/scripts/install.sh | sh
```

PowerShell (Windows):

```powershell
irm https://raw.githubusercontent.com/ink0rr/rgl/main/scripts/install.ps1 | iex
```

### Uninstall

Delete the `~/.rgl` directory.

## Compatibility

- The Shell installer can be used on Windows with [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/about), [MSYS](https://www.msys2.org) or equivalent set of tools.

## Known Issues

### unzip is required

The program [`unzip`](https://linux.die.net/man/1/unzip) is a requirement for the Shell installer.

**When does this issue occur?**

During the `install.sh` process, `unzip` is used to extract the zip archive.

**How can this issue be fixed?**

You can install unzip via `brew install unzip` on MacOS or `apt-get install unzip -y` on Linux.
