# rgl

Fast and minimal implementation of Regolith.

## Benchmark

Benchmark result on a project with ~2300 files, no filters, and a total size of 9.2MB.

```
$ hyperfine --warmup 2 --runs 10 --setup 'rgl clean' 'regolith run build' 'rgl run build' 'rgl run build --cached'

Benchmark 1: regolith run build
  Time (mean ± σ):      1.277 s ±  0.019 s    [User: 0.028 s, System: 0.270 s]
  Range (min … max):    1.256 s …  1.309 s    10 runs

Benchmark 2: rgl run build
  Time (mean ± σ):     278.0 ms ±  12.6 ms    [User: 9.4 ms, System: 135.3 ms]
  Range (min … max):   260.8 ms … 306.8 ms    10 runs

Benchmark 3: rgl run build --cached
  Time (mean ± σ):      69.2 ms ±   0.8 ms    [User: 3.1 ms, System: 54.1 ms]
  Range (min … max):    68.2 ms …  70.4 ms    10 runs

Summary
  rgl run build --cached ran
    4.02 ± 0.19 times faster than rgl run build
   18.46 ± 0.34 times faster than regolith run build
```

Results may vary depending on your machine, project size, and what kind of filters are used. In this case, rgl is 4x faster than Regolith and 18x faster when using the `--cached` flag.

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
