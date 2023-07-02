export abstract class Filter {
  abstract run(args?: string[], settings?: Record<string, unknown>): Promise<void>;
}
