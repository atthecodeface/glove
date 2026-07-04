import { Project } from "./project.js";
import { Log } from "./log.js";
import { WasmMemory } from "./wasm_memory.js";

export interface Application {
  wasm_memory: WasmMemory;
  project_load_completed(success: boolean): void;
  project_save_completed(success: boolean): void;
  thumbnails_updated(): void;
  current_project_name(): string | null;
  current_project(): Project;
  logger(): Log;
  get_resizable_content_size(): [number, number];
  set_view_needs_update(): void;
}
