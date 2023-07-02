import { join } from "../../deps.ts";

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

/**
 * @deprecated
 */
export function getDevelopmentPaths(name: string) {
  const comMojang = findComMojangPath();
  return {
    developmentBehaviorPacks: join(comMojang, "development_behavior_packs", `${name}_bp`),
    developmentResourcePacks: join(comMojang, "development_resource_packs", `${name}_rp`),
  };
}
