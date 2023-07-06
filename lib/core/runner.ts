import { copy, debounce, emptyDir, join, resolve } from "../../deps.ts";
import { rmdir } from "../utils/fs.ts";
import { logger } from "../utils/logger.ts";
import { getProjectConfig } from "./config.ts";
import { createContext } from "./context.ts";
import { exportProject } from "./export.ts";
import { runProfile } from "./profile.ts";

export async function runOrWatch(profileName: string, watch?: boolean) {
  const config = await getProjectConfig();
  const profile = config.regolith.profiles.get(profileName);
  if (!profile) {
    throw Error(`Profile "${profileName}" does not exist!`);
  }

  logger.info(`Running "${profileName}" profile.`);

  const [context, disposeContext] = await createContext(config, profileName, profile.export);

  await emptyDir(context.temp);

  await Promise.all([
    copy(context.packs.behaviorPack, join(context.temp, "BP")),
    copy(context.packs.resourcePack, join(context.temp, "RP")),
    Deno.symlink(resolve(context.dataPath), join(context.temp, "data"), { type: "dir" }),
  ]);
  await runProfile(profile);
  await exportProject();

  // Clean up
  await rmdir(context.temp);
  logger.info(`Successfully ran the "${profileName}" profile.`);
  disposeContext();

  if (watch) {
    const watcher = Deno.watchFs([
      config.regolith.dataPath,
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
