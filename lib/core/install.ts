import { copy, exists, join, semver } from "../../deps.ts";
import { logger } from "../utils/logger.ts";
import { getFilterCacheDir } from "./cache.ts";
import { ProjectConfig } from "./config.ts";
import { resolveURL } from "./resolver.ts";

export async function installFilter(config: ProjectConfig, filter: string) {
  let name: string;
  let url: string;
  let version: string | undefined;

  if (filter.includes("==")) {
    const splitted = filter.split("==");
    if (splitted.length != 2) {
      throw new Error(`Invalid filter name: ${filter}`);
    }
    [url, version] = splitted;
  } else {
    url = filter;
  }

  if (url.includes("/")) {
    const splitted = url.split("/");
    name = splitted.pop()!;
    url = splitted.join("/");
    if (url.match(/^hhtps?:\/\//i)) {
      throw new Error(`Invalid URL: ${url}`);
    }
  } else {
    name = url;
    url = resolveURL(name);
  }

  const ref = await getFilterRef(name, url, version);
  if (!ref) {
    throw new Error(`Unable to find version of the filter that satisfies the specified constraints.`);
  }
  await downloadFilter(name, url, ref);

  config.regolith.filterDefinitions.set(name, {
    url,
    version: version ?? ref,
  });
}

export async function downloadFilter(name: string, url: string, ref: string, force = true) {
  await Deno.mkdir(join(".regolith", "cache", "filters"), { recursive: true }).catch(() => {});

  const filterDir = join(".regolith", "cache", "filters", name);
  if (await exists(filterDir) && !force) {
    throw new Error(`Filter "${name}" already exists. Use --force to overwrite.`);
  } else {
    await Deno.remove(filterDir, { recursive: true }).catch(() => {});
  }

  logger.info(`Downloading filter "${name}"...`);
  logger.info(`Filter "${name}" resolved to ${url}`);

  const httpUrl = `https://${url}`;
  const cache = await getFilterCacheDir(httpUrl);

  if (!await exists(cache)) {
    await Deno.mkdir(cache, { recursive: true });
    await new Deno.Command("git", {
      args: ["clone", httpUrl, "."],
      cwd: cache,
    }).output();
  }
  logger.info(`Checking out "${ref}"...`);
  await new Deno.Command("git", {
    args: ["checkout", ref],
    cwd: cache,
  }).output();

  await copy(join(cache, name), filterDir);
  // TODO: Install filter dependencies

  logger.info(`Filter "${name}" downloaded successfully.`);
}

async function getFilterRef(name: string, url: string, version?: string) {
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
