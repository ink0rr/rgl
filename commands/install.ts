import { Command, semver } from "../deps.ts";
import { Config } from "../lib/core/config.ts";
import { downloadFilter, installFilter } from "../lib/core/install.ts";

export const install = new Command()
  .name("install")
  .alias("i")
  .description(
    `Downloads and installs Regolith filters from the internet, and adds them to the "filterDefinitions" list of the project's "config.json" file.`,
  )
  .arguments("[...filters:string]")
  .action(async (_, ...filters) => {
    const config = await Config.load();
    if (filters.length) {
      for (const filter of filters) {
        await installFilter(config, filter);
      }
      await config.save();
    } else {
      for (const [name, def] of config.filterDefinitions) {
        if (def.runWith !== undefined) continue;
        const ref = semver.valid(def.version) ? `${name}-${def.version}` : def.version;
        await downloadFilter(name, def.url, ref);
      }
    }
  });
