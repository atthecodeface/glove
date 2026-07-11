import { Project } from "./project.js";
import { Log } from "./log.js";
import { WasmMemory } from "./wasm_memory.js";

import { WebglCanvasClient } from "./webgl_canvas.js";

export interface Application {
  wasm_memory: WasmMemory;
  add_tab(
    application_tab: ApplicationTab,
    web_canvas_client: WebglCanvasClient | null,
  ): void;
  project_load_completed(success: boolean): void;
  project_save_completed(success: boolean): void;
  thumbnails_updated(): void;
  current_project_name(): string | null;
  current_project(): Project;
  logger(): Log;
  get_resizable_content_size(): [number, number];
  set_view_needs_update(): void;
}

export interface ApplicationTab {
  tab_name(): string;
  tab_text(): string;
  tab_selected(): void;
  tab_deselected(): void;
}
