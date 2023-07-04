import { copy, join } from "../../deps.ts";
import { Profile } from "../schemas/profile.ts";
import { logger } from "../utils/logger.ts";
import { findMojangDir } from "../utils/mojang_dir.ts";
import { ProjectConfig } from "./config.ts";

async function getExportPaths(name: string, profile: Profile) {
  const target = profile.export.target;
  switch (target) {
    case "development": {
      const mojangDir = await findMojangDir();
      return {
        bpPath: join(mojangDir, "development_behavior_packs", `${name}_bp`),
        rpPath: join(mojangDir, "development_resource_packs", `${name}_rp`),
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
  const { bpPath, rpPath } = await getExportPaths(config.name, profile);

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

export async function exportFrom(from: string, to: { bpPath: string; rpPath: string }) {
  await Promise.all([
    Deno.remove(to.bpPath, { recursive: true }).catch(() => {}),
    Deno.remove(to.rpPath, { recursive: true }).catch(() => {}),
  ]);

  logger.info(`Moving files to target location: 
      BP: ${to.bpPath}
      RP: ${to.rpPath}`);

  try {
    await Promise.all([
      Deno.rename(join(from, "BP"), to.bpPath),
      Deno.rename(join(from, "RP"), to.rpPath),
    ]);
  } catch {
    await Promise.all([
      copy(join(from, "BP"), to.bpPath),
      copy(join(from, "RP"), to.rpPath),
    ]);
  }
}
