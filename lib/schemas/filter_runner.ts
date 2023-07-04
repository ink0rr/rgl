import { z } from "../../deps.ts";

export const filterRunnerSchema = z.object({
  disabled: z.boolean().optional(),
  filter: z.string().optional(),
  profile: z.string().optional(),
  settings: z.record(z.unknown()).optional(),
  arguments: z.array(z.string()).optional(),
});

export type FilterRunner = z.infer<typeof filterRunnerSchema>;
