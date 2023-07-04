import { z } from "../../deps.ts";

export const filterDefinitionSchema = z.object({
  runWith: z.string().optional(),
  script: z.string().optional(),
  command: z.string().optional(),
  url: z.string().optional(),
  version: z.string().optional(),
}).transform((value) => {
  if (value.runWith) {
    return z.object({
      runWith: z.string(),
      script: z.string().optional(),
      command: z.string().optional(),
    }).parse(value);
  }

  return z.object({
    runWith: z.undefined(),
    url: z.string(),
    version: z.string(),
  }).parse(value);
});

export type FilterDefinition = z.infer<typeof filterDefinitionSchema>;
