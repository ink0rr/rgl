import { Profile } from "../schemas/profile.ts";
import { logger } from "../utils/logger.ts";
import { useContext } from "./context.ts";
import { runFilter } from "./filter.ts";

export async function runProfile(profile: Profile) {
  const context = useContext();
  for (const entry of profile.filters) {
    if (entry.disabled) {
      logger.info(`Filter "${entry.filter}" is disabled, skipping...`);
      continue;
    }
    if (entry.profile === context.currentProfile) {
      throw new Error(`Found circular dependency in profile "${context.currentProfile}"`);
    }
    if (entry.profile) {
      const profile = context.profiles.get(entry.profile);
      if (!profile) {
        throw new Error(`Profile "${entry.profile}" does not exist in profiles`);
      }
      logger.info(`Running "${entry.profile}" profile filters`);
      await runProfile(profile);
      continue;
    }

    const name = entry.filter;
    if (name) {
      const def = context.filterDefinitions.get(name);
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
