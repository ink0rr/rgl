import { Command } from "../deps.ts";
import { getUserConfig } from "../lib/core/config.ts";
import { logger } from "../lib/utils/logger.ts";

export const config = new Command()
  .name("config")
  .description("Runs Regolith using specified profile")
  .arguments("[key:string] [value:string]")
  .option("-d, --delete", "Delete a key")
  .action(async (options, key, value) => {
    const userConfig = await getUserConfig();
    if (!key) {
      logger.info(`User config values:
${JSON.stringify(userConfig, null, 6).slice(2, -2)}`);
      return;
    }
    if (options.delete) {
      delete userConfig[key];
      logger.info(`Delete user config: ${key}`);
    } else if (value) {
      userConfig[key] = value;
      logger.info(`Set user config:
      "${key}": "${value}"`);
    } else {
      logger.info(`User config value:
      "${key}": "${value}"`);
    }
    await userConfig.save();
  });
