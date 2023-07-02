import { copy, join } from "../../deps.ts";
import { logger } from "../utils/logger.ts";
import { loadConfig } from "./config.ts";
import { Filter } from "./filter.ts";
import { DenoFilter } from "./filter_deno.ts";
import { RemoteFilter } from "./filter_remote.ts";

export async function runProfile(profileName: string) {
  const config = await loadConfig();
  const profile = config.profiles.get(profileName);

  if (!profile) {
    throw new Error(`Profile "${profileName}" not found`);
  }

  // Initialze tmp dir
  const tmp = "./.regolith/tmp";
  await Deno.remove(tmp, { recursive: true }).catch(() => {});
  await Deno.mkdir(tmp, { recursive: true }).catch(() => {});

  const { behaviorPack, resourcePack } = config.packs;
  await Promise.all([
    copy(behaviorPack, join(tmp, "BP")),
    copy(resourcePack, join(tmp, "RP")),
  ]);

  for (const entry of profile.filters) {
    if (entry.disabled) {
      logger.info(`Filter "${entry.filter}" is disabled, skipping...`);
      continue;
    }
    if (entry.filter) {
      const filterDefinition = config.filterDefinitions.get(entry.filter);
      if (!filterDefinition) {
        logger.warn(`Filter "${entry.filter}" does not exist in filterDefinitions`);
        continue;
      }

      let filter: Filter;
      switch (filterDefinition.runWith) {
        case "deno":
          filter = new DenoFilter(entry.filter, filterDefinition.script);
          break;
        case "":
          filter = new RemoteFilter();
          break;
        default:
          throw Error("Invalid filter definition");
      }
      filter.run(entry.arguments, entry.settings);
    }
  }
}
