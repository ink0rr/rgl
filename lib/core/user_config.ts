import { join } from "../../deps.ts";

/**
 * Lousy port of Go `os.UserCacheDir()`
 *
 * https://pkg.go.dev/os#UserCacheDir
 */
function getUserCacheDir() {
  let dir: string | undefined;
  switch (Deno.build.os) {
    case "windows":
      dir = Deno.env.get("LocalAppData");
      if (!dir) {
        throw Error("%LocalAppData% is not defined");
      }
      break;
    case "darwin":
      dir = Deno.env.get("HOME");
      if (!dir) {
        throw Error("$HOME is not defined");
      }
      dir += "/Library/Caches";
      break;
    default:
      dir = Deno.env.get("XDG_CACHE_HOME") ?? Deno.env.get("HOME");
      if (!dir) {
        throw Error("Neither $XDG_CACHE_HOME nor $HOME are defined");
      }
      dir += "/.cache";
      break;
  }
  return dir;
}

function _getUserConfigPath() {
  return join(getUserCacheDir(), "regolith", "user_config.json");
}
