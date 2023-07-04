import { copy, exists, join, semver } from "../../deps.ts";
import { getFilterCacheDir } from "./cache.ts";

export async function installRemoteFilter(name: string, url: string, ref: string, force = true) {
  const filterDir = join(".regolith", "cache", "filters", name);
  if (await exists(filterDir) && !force) {
    throw new Error(`Filter "${name}" already exists. Use --force to overwrite.`);
  } else {
    await Deno.remove(filterDir, { recursive: true }).catch(() => {});
  }

  const httpUrl = `https://${url}`;
  const cache = await getFilterCacheDir(httpUrl);

  if (!await exists(cache)) {
    await Deno.mkdir(cache, { recursive: true });
    await new Deno.Command("git", {
      args: ["clone", httpUrl, "."],
      cwd: cache,
    }).output();
  }

  await new Deno.Command("git", {
    args: ["checkout", ref],
    cwd: cache,
  }).output();

  await copy(join(cache, name), filterDir);
  // TODO: Install filter dependencies
}

export async function getFilterRef(name: string, url: string, version?: string) {
  if (version && semver.valid(version)) {
    return `${name}-${version}`;
  }

  const decoder = new TextDecoder();
  if (!version || version === "latest") {
    const process = await new Deno.Command("git", {
      args: ["ls-remote", "--tags", `https://${url}`],
    }).output();

    const tags = [];
    for (const line of decoder.decode(process.stdout).split("\n")) {
      const tag = line.split(`refs/tags/${name}-`).at(1);
      if (tag && semver.valid(tag)) {
        tags.push(tag);
      }
    }
    const tag = semver.sort(tags).at(-1);
    if (tag) {
      return `${name}-${tag}`;
    }
  }

  if (!version || version === "HEAD") {
    const process = await new Deno.Command("git", {
      args: ["ls-remote", "--symref", `https://${url}`, "HEAD"],
    }).output();

    const sha = decoder.decode(process.stdout).split("\n").at(1)?.split("\t").at(0);
    return sha;
  }
}
