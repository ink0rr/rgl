import { copy, debounce, join } from "../../deps.ts";
import { logger } from "../utils/logger.ts";
import { ProjectConfig } from "./project_config.ts";
import { exportProject } from "./export.ts";
import { runProfile } from "./profile.ts";

export async function runOrWatch(profileName: string, watch?: boolean) {
  const config = await ProjectConfig.load();
  const profile = config.profiles.get(profileName);
  if (!profile) {
    throw Error(`Profile "${profileName}" does not exist!`);
  }

  logger.info(`Running "${profileName}" profile.`);

  const tmp = "./.regolith/tmp";
  await Deno.remove(tmp, { recursive: true }).catch(() => {});
  await Deno.mkdir(tmp, { recursive: true }).catch(() => {});

  const { behaviorPack, resourcePack } = config.packs;
  await Promise.all([
    copy(behaviorPack, join(tmp, "BP")),
    copy(resourcePack, join(tmp, "RP")),
    copy(config.dataPath, join(tmp, "data")),
  ]);

  await runProfile(config, profile);
  await exportProject(config, profile);
  logger.info(`Successfully ran the "${profileName}" profile.`);

  if (watch) {
    const watcher = Deno.watchFs([
      config.dataPath,
      config.packs.behaviorPack,
      config.packs.resourcePack,
    ]);

    const restart = debounce(async () => {
      logger.warn("Restarting...");
      watcher.close();
      await runOrWatch(profileName, true);
    }, 100);

    logger.info("Watching for changes...");
    logger.info("Press Ctrl+C to stop watching.");

    for await (const _ of watcher) {
      restart();
    }
  }
}
