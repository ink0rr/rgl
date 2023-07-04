import { join } from "../../deps.ts";
import { projectConfigSchema } from "../schemas/project_config.ts";
import { userConfigSchema } from "../schemas/user_config.ts";
import { readJson, writeJson } from "../utils/fs.ts";
import { getRegolithCacheDir } from "./cache.ts";

export async function getProjectConfig() {
  const config = await readJson("./config.json").then(projectConfigSchema.parse);
  return Object.assign(config, {
    async save() {
      await writeJson("./config.json", config, (_, v) => {
        if (v instanceof Map) {
          return Object.fromEntries(v);
        }
        return v;
      });
    },
  });
}

export type ProjectConfig = Awaited<ReturnType<typeof getProjectConfig>>;

export async function getUserConfig() {
  const config = await readJson(join(getRegolithCacheDir(), "user_config.json")).then(userConfigSchema.parse);
  return Object.assign(config, {
    async save() {
      await writeJson("./user_config.json", config);
    },
  });
}

export type UserConfig = Awaited<ReturnType<typeof getUserConfig>>;
