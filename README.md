# rgl

Fast and minimal implementation of Regolith.

Portions of this code are based on [minecraft-debugger](https://github.com/Mojang/minecraft-debugger), which is licensed under the MIT License.

Copyright (c) Microsoft Corporation

## Benchmark

Benchmark result on a project with ~2700 files, no filters, and a total size of 14MB.

```
$ hyperfine --warmup 3 --runs 10 --setup 'rgl clean' 'regolith run' 'regolith run --experiments size_time_check' 'rgl run' 'rgl run --cached'

Benchmark 1: regolith run
  Time (mean ± σ):      1.543 s ±  0.017 s    [User: 0.037 s, System: 0.427 s]
  Range (min … max):    1.511 s …  1.577 s    10 runs

Benchmark 2: regolith run --experiments size_time_check
  Time (mean ± σ):     438.5 ms ±   7.2 ms    [User: 15.2 ms, System: 62.5 ms]
  Range (min … max):   429.1 ms … 448.7 ms    10 runs

Benchmark 3: rgl run
  Time (mean ± σ):     384.8 ms ±  10.4 ms    [User: 44.8 ms, System: 614.1 ms]
  Range (min … max):   364.1 ms … 402.8 ms    10 runs

Benchmark 4: rgl run --cached
  Time (mean ± σ):      76.1 ms ±   1.2 ms    [User: 10.6 ms, System: 46.9 ms]
  Range (min … max):    74.0 ms …  77.8 ms    10 runs

Summary
  rgl run --cached ran
    5.06 ± 0.16 times faster than rgl run
    5.76 ± 0.13 times faster than regolith run --experiments size_time_check
   20.28 ± 0.39 times faster than regolith run
```

Results may vary depending on your machine, project size, and what kind of filters are used. In this case, rgl is 4x faster than Regolith and 20x faster when using the `--cached` flag.

### Install

Shell (Mac, Linux):

```sh
curl -fsSL rgl.ink0rr.dev/install.sh | sh
```

PowerShell (Windows):

```powershell
irm rgl.ink0rr.dev/install.ps1 | iex
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
