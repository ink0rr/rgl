# rgl

Fast and efficient Bedrock Addon Compiler.

## Benchmark

Benchmark result on the [bedrock-samples](https://github.com/Mojang/bedrock-samples) repo.

```
$ hyperfine --warmup 2 --runs 10 --setup 'rgl clean' 'regolith run --experiments=size_time_check' 'rgl run'

Benchmark 1: regolith run --experiments=size_time_check
  Time (mean ± σ):      2.512 s ±  0.018 s    [User: 0.481 s, System: 2.053 s]
  Range (min … max):    2.487 s …  2.544 s    10 runs

Benchmark 2: rgl run
  Time (mean ± σ):     333.6 ms ±  10.1 ms    [User: 753.1 ms, System: 2103.4 ms]
  Range (min … max):   325.5 ms … 358.8 ms    10 runs

Summary
  rgl run ran
    7.53 ± 0.23 times faster than regolith run --experiments=size_time_check
```

Results may vary depending on your machine, project size, and what kind of filters are used. In this case, rgl is 7x faster than Regolith with size_time_check experiment enabled.

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
