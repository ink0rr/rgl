import { join } from "../../deps.ts";
import { FilterDefinition } from "../schemas/filter_definition.ts";
import { Profile } from "../schemas/profile.ts";
import { findMojangDir } from "../utils/mojang_dir.ts";
import { ProjectConfig } from "./config.ts";

export type RunContext = {
  name: string;
  packs: ProjectConfig["packs"];
  dataPath: string;
  export: Profile["export"];
  currentProfile: string;
  profiles: Map<string, Profile>;
  filterDefinitions: Map<string, FilterDefinition>;
  temp: string;
};

let context: RunContext | undefined;

export async function createContext(config: ProjectConfig, profileName: string, $export: Profile["export"]) {
  if (context) {
    throw new Error("Context already exists! Did you forget to dispose it?");
  }

  let temp;
  if ($export.target === "development") {
    temp = join(await findMojangDir(), ".regolith");
  } else {
    temp = join(".regolith", "tmp");
  }

  context = {
    name: config.name,
    packs: config.packs,
    dataPath: config.regolith.dataPath,
    export: $export,
    currentProfile: profileName,
    profiles: config.regolith.profiles,
    filterDefinitions: config.regolith.filterDefinitions,
    temp,
  };

  return [context, () => {
    context = undefined;
  }] as const;
}

export function useContext() {
  if (!context) {
    throw new Error("createContext() must be called before useContext()!");
  }
  return context;
}
