import { logger } from "../utils/logger.ts";
import { Config, Profile } from "./config.ts";
import { Filter } from "./filter.ts";
import { DenoFilter } from "./filter_deno.ts";
import { RemoteFilter } from "./filter_remote.ts";

export async function runProfile(config: Config, profile: Profile) {
  for (const entry of profile.filters) {
    if (entry.disabled) {
      logger.info(`Filter "${entry.filter}" is disabled, skipping...`);
      continue;
    }
    if (entry.profile) {
      const profile = config.profiles.get(entry.profile);
      if (!profile) {
        throw Error(`Profile "${entry.profile}" does not exist in profiles`);
      }
      logger.info(`Running "${entry.profile}" profile filters`);
      await runProfile(config, profile);
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
          filter = new RemoteFilter(entry.filter);
          break;
        default:
          throw Error(`Invalid filter definition: ${entry.filter}`);
      }
      logger.info(`Running filter "${entry.filter}"`);
      await filter.run(entry.arguments, entry.settings);
    }
  }
}
