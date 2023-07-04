import { copy, join } from "../../deps.ts";
import { logger } from "../utils/logger.ts";
import { ProjectConfig, Profile } from "./project_config.ts";

function findComMojangPath() {
  if (Deno.build.os === "windows") {
    const localAppData = Deno.env.get("LOCALAPPDATA");
    if (!localAppData) {
      throw new Error("LOCALAPPDATA not found");
    }
    const comMojangPath = join(
      localAppData,
      "Packages",
      "Microsoft.MinecraftUWP_8wekyb3d8bbwe",
      "LocalState",
      "games",
      "com.mojang",
    );
    return comMojangPath;
  }

  throw new Error("Not implemented");
}

function getExportPaths(name: string, profile: Profile) {
  const target = profile.export.target;
  switch (target) {
    case "development": {
      const comMojang = findComMojangPath();
      return {
        bpPath: join(comMojang, "development_behavior_packs", `${name}_bp`),
        rpPath: join(comMojang, "development_resource_packs", `${name}_rp`),
      };
    }
    case "exact":
      return {
        bpPath: profile.export.bpPath,
        rpPath: profile.export.rpPath,
      };
    case "local":
      return {
        bpPath: "./build/BP",
        rpPath: "./build/RP",
      };
    default:
      throw new Error(`Unsupported export target: ${target}`);
  }
}

export async function exportProject(config: ProjectConfig, profile: Profile) {
  const { bpPath, rpPath } = getExportPaths(config.name, profile);

  await Promise.all([
    Deno.remove(bpPath, { recursive: true }).catch(() => {}),
    Deno.remove(rpPath, { recursive: true }).catch(() => {}),
  ]);

  logger.info(`Moving files to target location: 
      BP: ${bpPath}
      RP: ${rpPath}`);

  try {
    await Promise.all([
      Deno.rename(join("./.regolith/tmp/BP"), bpPath),
      Deno.rename(join("./.regolith/tmp/RP"), rpPath),
    ]);
  } catch {
    await Promise.all([
      copy(join("./.regolith/tmp/BP"), bpPath),
      copy(join("./.regolith/tmp/RP"), rpPath),
    ]);
  }
}
