import { join } from "../../deps.ts";
import { readJson } from "../utils/fs.ts";
import { getRegolithCacheDir } from "./cache.ts";

type Resolver = {
  filters: Record<string, { url: string }>;
};

const defaultResolver = await readJson<Resolver>(join(getRegolithCacheDir(), "resolvers", "resolver_0.json"));

export function resolveURL(name: string) {
  const resolver = new Map(Object.entries(defaultResolver.filters));

  const filter = resolver.get(name);
  if (!filter) {
    throw Error(`Filter "${name}" not found`);
  }

  return filter.url;
}
