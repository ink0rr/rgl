import { exists, join } from "../../deps.ts";
import { getUserConfig } from "../core/config.ts";

export async function findMojangDir() {
  let dir;
  if (Deno.build.os === "windows") {
    const localAppData = Deno.env.get("LOCALAPPDATA");
    if (!localAppData) {
      throw new Error("LOCALAPPDATA not found");
    }
    dir = join(
      localAppData,
      "Packages",
      "Microsoft.MinecraftUWP_8wekyb3d8bbwe",
      "LocalState",
      "games",
      "com.mojang",
    );
  } else {
    const { mojang_dir } = await getUserConfig();
    if (!mojang_dir) {
      throw new Error(
        `Non-windows machine detected, you have to specify your own com.mojang folder using "rgl config --mojang-dir=<DIR>"`,
      );
    }
    dir = mojang_dir;
  }
  if (!await exists(dir)) {
    throw new Error(`Directory ${dir} does not exists`);
  }
  return dir;
}
