import { z } from "../../deps.ts";
import { filterDefinitionSchema } from "./filter_definition.ts";

export const remoteFilterSchema = z.object({
  filters: z.array(filterDefinitionSchema),
  version: z.string().optional(),
});

export type RemoteFilter = z.infer<typeof remoteFilterSchema>;
