import { copy, exists, join, semver } from "../../deps.ts";
import { remoteFilterSchema } from "../schemas/remote_filter.ts";
import { readJson, writeJson } from "../utils/fs.ts";
import { logger } from "../utils/logger.ts";
import { getFilterCacheDir } from "./cache.ts";
import { getFilter } from "./filter.ts";

export async function installRemoteFilter(name: string, url: string, ref: string, force = true) {
  const filterDir = join(".regolith", "cache", "filters", name);
  if (await exists(filterDir) && !force) {
    throw new Error(`Filter "${name}" already exists. Use --force to overwrite.`);
  } else {
    await Deno.remove(filterDir, { recursive: true }).catch(() => {});
  }

  logger.info(`Installing filter "${name}"...`);

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

  // Add version to filter.json
  const filterConfigPath = join(filterDir, "filter.json");
  const filterConfig = await readJson(filterConfigPath).then(remoteFilterSchema.parse);
  filterConfig.version = refToVersion(ref);
  await writeJson(filterConfigPath, filterConfig);

  // Install filter dependencies
  for (const entry of filterConfig.filters) {
    // TODO: Handle requirements field
    const filter = getFilter(entry.runWith);
    if (filter.installDependencies) {
      logger.info(`Installing dependencies for "${name}"...`);
      await filter.installDependencies(filterDir);
    }
  }
  logger.info(`Filter "${name}" installed successfully.`);
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

export function refToVersion(ref: string) {
  return semver.valid(ref.split("-")[1]) ?? ref;
}
