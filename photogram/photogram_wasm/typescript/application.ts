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

  /** Indicate that the project has been updated so the MappedNps may have changed, divs may need to be updated, etc
   *
   * In the next 'tick' the tab_project_updated() will be invoked for the selected tab
   */
  set_project_updated(): void;

  /** Indicate that a redraw is required perhaps because of smaller changes such as drags, resize, etc
   *
   * In the next 'tick' the tab_resize() and tab_redraw() / webgl_redraw() will be invoked for the selected tab as required
   */
  set_redraw_required(): void;
}

export interface ApplicationTab {
  tab_name(): string;
  tab_text(): string;
  /** Invoked when another tab is selected and this was selected */
  tab_deselected(): void;
  /** Invoked when the tab is selected, after the previous is deselected */
  tab_selected(): void;
  /** Invoked the current_project has changed; all clients of any previous project will have been dropped */
  tab_project_selected(p: Project): void;
  /** Invoked after a new tab is selected or after a set of changes have occurred, in an 'AnimationFrame' after set_project_updated */
  tab_project_updated(): void;
  /** Invoked when the tab is resized, in an 'AnimationFrame' after set_redraw_required (or set_project_updated) */
  tab_resize(w: number, h: number): void;
  /** Invoked in an 'AnimationFrame' after set_redraw_required (after any resize) (or set_project_updated) */
  tab_redraw(): void;
}
