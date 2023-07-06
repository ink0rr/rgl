import { Command, semver } from "../deps.ts";
import { getProjectConfig } from "../lib/core/config.ts";
import { getFilterRef, installRemoteFilter, refToVersion } from "../lib/core/remote_filter.ts";
import { resolveURL } from "../lib/core/resolver.ts";
import { logger } from "../lib/utils/logger.ts";

export const install = new Command()
  .name("install")
  .alias("i")
  .description(
    `Downloads and installs Regolith filters from the internet, and adds them to the "filterDefinitions" list of the project's "config.json" file.`,
  )
  .arguments("[...filters:string]")
  .action(async (_, ...filters) => {
    const config = await getProjectConfig();
    if (filters.length) {
      for (const filter of filters) {
        const { name, url, version } = parseFilterInfo(filter);

        logger.info(`Installing filter "${name}"...`);
        const ref = await getFilterRef(name, url, version);
        if (!ref) {
          throw new Error(`Unable to find version of the filter that satisfies the specified constraints.`);
        }
        logger.info(`Resolved filter "${name}" to ${url}@${ref}`);
        await installRemoteFilter(name, url, ref);
        config.regolith.filterDefinitions.set(name, {
          url,
          version: refToVersion(ref),
        });
        logger.info(`Filter "${name}" installed successfully.`);
      }
      await config.save();
    } else {
      for (const [name, def] of config.regolith.filterDefinitions) {
        if (def.runWith !== undefined) continue;
        const ref = semver.valid(def.version) ? `${name}-${def.version}` : def.version;
        await installRemoteFilter(name, def.url, ref);
      }
    }
  });

function parseFilterInfo(filter: string) {
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
  return { name, url, version };
}
