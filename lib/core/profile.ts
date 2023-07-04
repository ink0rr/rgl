import { logger } from "../utils/logger.ts";
import { Config, Profile } from "./config.ts";
import { runFilter } from "./filter.ts";

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

    const name = entry.filter;
    if (name) {
      const def = config.filterDefinitions.get(name);
      if (!def) {
        logger.warn(`Filter "${name}" does not exist in filterDefinitions`);
        continue;
      }

      logger.info(`Running filter "${name}"`);
      const runArgs = entry.arguments ?? [];
      if (entry.settings) {
        runArgs.unshift(JSON.stringify(entry.settings));
      }
      await runFilter(name, runArgs, def);
    }
  }
}
