import { join } from "../../deps.ts";
import { Profile } from "./config.ts";

export function getExportPaths(name: string, profile: Profile) {
  const target = profile.export.target;
  switch (target) {
    case "development":
      return getDevelopmentPaths(name);
    case "local":
      return {
        bp: "./build/BP",
        rp: "./build/RP",
      };
    default:
      throw new Error(`Unknown export target: ${target}`);
  }
}

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

function getDevelopmentPaths(name: string) {
  const comMojang = findComMojangPath();
  return {
    bp: join(comMojang, "development_behavior_packs", `${name}_bp`),
    rp: join(comMojang, "development_resource_packs", `${name}_rp`),
  };
}

export function exportProject(){
  
}
