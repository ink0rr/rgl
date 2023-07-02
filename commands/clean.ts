import { Command } from "../deps.ts";
import { Config } from "../lib/core/config.ts";
import { logger } from "../lib/utils/logger.ts";
import { getDevelopmentPaths } from "../lib/utils/paths.ts";

export const clean = new Command()
  .name("clean")
  .description("Clean")
  .action(async () => {
    const config = await Config.load();

    logger.info("Cleaning development export paths...");
    const { developmentBehaviorPacks, developmentResourcePacks } = getDevelopmentPaths(config.name);
    await Promise.all([
      Deno.remove(developmentBehaviorPacks, { recursive: true }),
      Deno.remove(developmentResourcePacks, { recursive: true }),
    ]);
    logger.success(`Deleted: ${developmentBehaviorPacks}`);
    logger.success(`Deleted: ${developmentResourcePacks}`);
  });
