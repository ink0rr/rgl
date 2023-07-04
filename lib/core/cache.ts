import { join, toHashString } from "../../deps.ts";

/**
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

export function getRegolithCacheDir() {
  return join(getUserCacheDir(), "regolith");
}

/**
 * @param url with the `https://` prefix
 */
export async function getFilterCacheDir(url: string) {
  const encoder = new TextEncoder();
  const md5 = await crypto.subtle.digest("MD5", encoder.encode(url));

  return join(getRegolithCacheDir(), "filter-cache", toHashString(md5));
}
