import { z } from "../../deps.ts";

const remoteFilterDefinitionSchema = z.object({
  name: z.string().optional(),
  runWith: z.string(),
  script: z.string().optional(),
  command: z.string().optional(),
  requirements: z.string().optional(),
});

export const remoteFilterSchema = z.object({
  filters: z.array(remoteFilterDefinitionSchema),
  version: z.string().optional(),
});

export type RemoteFilter = z.infer<typeof remoteFilterSchema>;
