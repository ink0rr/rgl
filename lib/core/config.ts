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

  // Is a remote filter
  return z.object({
    runWith: z.undefined(),
    url: z.string(),
    version: z.string(),
  }).parse(value);
});

const runFilterSchema = z.object({
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
    }),
    z.object({
      target: z.literal("exact"),
      bpPath: z.string(),
      rpPath: z.string(),
    }),
    z.object({
      target: z.literal("local"),
    }),
  ]),
  filters: z.array(runFilterSchema).optional().default([]),
});

const configSchema = z.object({
  $schema: z.string().optional(),
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

export type FilterDefinition = z.infer<typeof filterDefinitionSchema>;

export type RunFilter = z.infer<typeof runFilterSchema>;

export type Profile = z.infer<typeof profileSchema>;

export class Config {
  private constructor(
    private config: z.infer<typeof configSchema>,
  ) {}

  get name() {
    return this.config.name;
  }

  get packs() {
    return this.config.packs;
  }

  get dataPath() {
    return this.config.regolith.dataPath;
  }

  get filterDefinitions() {
    return this.config.regolith.filterDefinitions;
  }

  get profiles() {
    return this.config.regolith.profiles;
  }

  async save() {
    await writeJson("./config.json", this.config, (_, value) => {
      if (value instanceof Map) {
        return Object.fromEntries(value);
      }
      return value;
    });
  }

  static async load() {
    const config = await readJson("./config.json").then(configSchema.parse);
    return new Config(config);
  }
}
