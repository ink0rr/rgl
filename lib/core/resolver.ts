import { crypto, exists, join, toHashString } from "../../deps.ts";
import { readJson } from "../utils/fs.ts";
import { getRegolithCacheDir } from "./cache.ts";

type Resolver = {
  filters: Record<string, { url: string }>;
};

async function getResolverCacheDir(httpUrl: string) {
  const encoder = new TextEncoder();
  const md5 = await crypto.subtle.digest("MD5", encoder.encode(httpUrl));

  return join(getRegolithCacheDir(), "filter-cache", toHashString(md5));
}

export async function getResolver() {
  // TODO: Get resolver from user config
  const resolverUrl = "https://github.com/Bedrock-OSS/regolith-filter-resolver";
  const cache = await getResolverCacheDir(resolverUrl);
  if (!await exists(cache)) {
    await Deno.mkdir(cache, { recursive: true });
    await new Deno.Command("git", {
      args: ["clone", resolverUrl, "."],
      cwd: cache,
    }).output();
  }

  await new Deno.Command("git", {
    args: ["pull"],
    cwd: cache,
  }).output();

  const resolver = await readJson<Resolver>(join(cache, "resolver.json"));
  return new Map(Object.entries(resolver.filters));
}

export async function resolveURL(name: string) {
  const resolver = await getResolver();

  const filter = resolver.get(name);
  if (!filter) {
    throw Error(`Filter "${name}" not found`);
  }

  return filter.url;
}
