import { z } from "../../deps.ts";
import { readJson, writeJson } from "../utils/fs.ts";

const filterDefinitionSchema = z.object({
  runWith: z.string().optional(),
  script: z.string().optional(),
  url: z.string().optional(),
  version: z.string().optional(),
}).transform((value) => {
  if (value.runWith) {
    return z.object({
      runWith: z.string(),
      script: z.string(),
    }).parse(value);
  }
  return z.object({
    runWith: z.literal("").optional().default(""),
    url: z.string(),
    version: z.string(),
  }).parse(value);
});

const filterSchema = z.object({
  disabled: z.boolean().optional(),
  filter: z.string().optional(),
  profile: z.string().optional(),
  settings: z.record(z.unknown()).optional(),
  arguments: z.array(z.string()).optional(),
});

const profileSchema = z.object({
  export: z.discriminatedUnion("target", [
    z.object({
      target: z.literal("development"),
      readOnly: z.boolean().optional(),
    }),
    z.object({
      target: z.literal("local"),
    }),
  ]),
  filters: z.array(filterSchema).optional().default([]),
});

const configSchema = z.object({
  author: z.string(),
  name: z.string(),
  packs: z.object({
    behaviorPack: z.string(),
    resourcePack: z.string(),
  }),
  regolith: z.object({
    dataPath: z.string(),
    filterDefinitions: z.record(filterDefinitionSchema).transform((value) => new Map(Object.entries(value))),
    profiles: z.record(profileSchema).transform((value) => new Map(Object.entries(value))),
  }),
});

type ConfigSchema = z.infer<typeof configSchema>;

export class Config {
  #config: ConfigSchema;
  constructor(config: ConfigSchema) {
    this.#config = config;
  }

  get name() {
    return this.#config.name;
  }

  get packs() {
    return this.#config.packs;
  }

  get dataPath() {
    return this.#config.regolith.dataPath;
  }

  get filterDefinitions() {
    return this.#config.regolith.filterDefinitions;
  }

  get profiles() {
    return this.#config.regolith.profiles;
  }

  async save() {
    await writeJson("./config.json", this.#config);
  }

  static async load() {
    const config = await readJson("./config.json").then(configSchema.parse);
    return new Config(config);
  }
}

export async function loadConfig(): Promise<Config> {
  const config = await readJson("./config.json").then(configSchema.parse);
  return new Config(config);
}
