import { z } from "../../deps.ts";
import { filterRunnerSchema } from "./filter_runner.ts";

export const profileSchema = z.object({
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
  filters: z.array(filterRunnerSchema).optional().default([]),
});

export type Profile = z.infer<typeof profileSchema>;
