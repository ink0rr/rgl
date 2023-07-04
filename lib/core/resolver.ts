import { join } from "../../deps.ts";
import { readJson } from "../utils/fs.ts";
import { getRegolithCacheDir } from "./user_config.ts";

const defaultResolver = await readJson<{ filters: Record<string, { url: string }> }>(
  join(getRegolithCacheDir(), "resolvers", "resolver_0.json"),
);

export function resolveURL(name: string) {
  const resolver = new Map(Object.entries(defaultResolver.filters));

  const filter = resolver.get(name);
  if (!filter) {
    throw Error(`Filter "${name}" not found`);
  }

  return filter.url;
}
