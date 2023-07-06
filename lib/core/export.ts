import { copy, join, move } from "../../deps.ts";
import { rmdir } from "../utils/fs.ts";
import { logger } from "../utils/logger.ts";
import { findMojangDir } from "../utils/mojang_dir.ts";
import { useContext } from "./context.ts";

export async function getExportPaths() {
  const context = useContext();
  const e = context.export;
  switch (e.target) {
    case "development": {
      const mojangDir = await findMojangDir();
      return {
        behaviorPack: join(mojangDir, "development_behavior_packs", `${context.name}_bp`),
        resourcePack: join(mojangDir, "development_resource_packs", `${context.name}_rp`),
      };
    }
    case "exact":
      return {
        behaviorPack: e.bpPath,
        resourcePack: e.rpPath,
      };
    case "local":
      return {
        behaviorPack: "./build/BP",
        resourcePack: "./build/RP",
      };
  }
}

export async function exportProject() {
  const context = useContext();
  const paths = await getExportPaths();

  await Promise.all([
    rmdir(paths.behaviorPack),
    rmdir(paths.resourcePack),
  ]);

  logger.info(`Moving files to target location: 
      BP: ${paths.behaviorPack}
      RP: ${paths.resourcePack}`);

  const temp = {
    behaviorPack: join(context.temp, "BP"),
    resourcePack: join(context.temp, "RP"),
  };

  try {
    await Promise.all([
      move(temp.behaviorPack, paths.behaviorPack),
      move(temp.resourcePack, paths.resourcePack),
    ]);
  } catch (err) {
    if (context.export.target === "development") {
      throw err;
    }
    await Promise.all([
      copy(temp.behaviorPack, paths.behaviorPack),
      copy(temp.resourcePack, paths.resourcePack),
    ]);
  }
}
