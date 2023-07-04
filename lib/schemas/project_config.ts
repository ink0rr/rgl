import { z } from "../../deps.ts";
import { filterDefinitionSchema } from "./filter_definition.ts";
import { profileSchema } from "./profile.ts";

export const projectConfigSchema = z.object({
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
