import { createConsola } from "../../deps.ts";

export const logger = createConsola({
  level: 4,
  formatOptions: {
    date: false,
  },
});
