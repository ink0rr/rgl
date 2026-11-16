# rgl

Fast and efficient Bedrock Addon Compiler.

## Benchmark

Benchmark result on the [bedrock-samples](https://github.com/Mojang/bedrock-samples) repo.

```
$ hyperfine --warmup 2 --runs 10 --setup 'rgl clean' 'regolith run' 'regolith run --unsafe --symlink-export' 'rgl run'

Benchmark 1: regolith run (10 runs)
                mean     ±       σ          min     …     max
  Wall Time    356.6 ms  ±    23.9 ms     323.3 ms  …   395.6 ms
  Memory        49.2 MiB ±     1.9 MiB     46.8 MiB …    52.0 MiB

Benchmark 2: regolith run --unsafe --symlink-export (10 runs)
                mean     ±       σ          min     …     max
  Wall Time    185.9 ms  ±    16.8 ms     160.8 ms  …   209.2 ms           -47.9%  (1.9x faster)
  Memory        46.7 MiB ±     2.5 MiB     43.4 MiB …    49.8 MiB           -5.0%

Benchmark 3: rgl run (10 runs)
                mean     ±       σ          min     …     max
  Wall Time     44.6 ms  ±     4.9 ms      37.8 ms  …    51.5 ms           -87.5%  (8.0x faster)
  Memory        25.7 MiB ±     0.2 MiB     25.2 MiB …    25.9 MiB          -47.8%  (1.9x less)
```

Results may vary depending on your machine, project size, and what kind of filters are used. In this case, rgl is 8x faster than Regolith and 4.2x faster than Regolith with `--unsafe --symlink-export`.

### Install

Shell (Mac, Linux):

```sh
curl -fsSL rgl.ink0rr.dev/install.sh | sh
```

PowerShell (Windows):

```powershell
irm rgl.ink0rr.dev/install.ps1 | iex
```

### Shell Completions

Generate a completion script with `rgl completions <shell>`:

```sh
# Bash: ~/.bashrc
eval "$(rgl completions bash)"
# Zsh: ~/.zshrc
source <(rgl completions zsh)
# Fish: ~/.config/fish/config.fish
rgl completions fish | source
```

```powershell
# PowerShell: $PROFILE
rgl completions powershell | Out-String | Invoke-Expression
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
