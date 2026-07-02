import { WasmCip, WasmProject } from "../pkg/photogram_wasm";

import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application } from "./application.js";
import { ProjectSet } from "./project_set.js";

export class Project {
  private application: Application;
  private project_set: ProjectSet;
  private log: Logger;
  locator: string | null = null;
  private wasm_project: WasmProject | null = null;
  private modified: boolean = false;

  private thumbnails: HTMLImageElement[] = [];
  constructor(application: Application, log: Logger, project_set: ProjectSet) {
    this.application = application;
    this.log = log;
    this.project_set = project_set;
    this.thumbnails = [];
  }

  get_cip_name(cip: number): string | null {
    const name = this.wasm_project?.cip_name(cip);
    return name ? name : null;
  }

  repopulate() {
    const project_locator = this.locator ? this.locator : "<no project open>";
    const nps_length = this.wasm_project
      ? this.wasm_project.nps.num_points().toString()
      : "<no project open>";

    HtmlElement.fold_all_of(".set-project-name", null, (a, e) => {
      e.ele.innerHTML = project_locator;
      return a;
    });
    HtmlElement.fold_all_of(".set-nps-length", null, (a, e) => {
      e.ele.innerHTML = nps_length;
      return a;
    });
  }

  is_modified(): boolean {
    return this.modified && this.thumbnails.length != 0;
  }

  load_project(locator: string) {
    this.locator = locator;
    this.modified = false;
    this.project_set.load_project(
      this.locator,
      this.project_loaded.bind(this),
      this.project_load_error.bind(this),
    );
  }

  project_loaded(wasm_project: WasmProject): void {
    this.wasm_project = wasm_project;
    this.application.project_load_completed(true);
  }

  project_load_error(e: string): void {
    this.log.error(e);

    this.locator = null;
    this.application.project_load_completed(false);
  }

  save_project(locator: string | null) {
    locator = locator ? locator : this.locator;
    if (this.wasm_project === null || locator === null) {
      this.application.project_save_completed(false);
      return;
    }
    this.project_set.save_project(
      locator,
      this.wasm_project,
      this.project_saved.bind(this),
      this.project_save_error.bind(this),
    );
  }

  project_saved(): void {
    this.modified = false;
    this.application.project_save_completed(true);
  }

  project_save_error(e: string): void {
    this.log.error(e);
    this.application.project_save_completed(false);
  }

  /** Fetch the thumbnails (for a server project; does not work for 'local')
   *
   */
  get_cip_by_name(name: string): WasmCip | null {
    if (this.wasm_project === null) {
      return null;
    }
    return this.wasm_project.cip(name);
  }
}
/*
//mp image_uri
get_image_uri(cip) {
    return `${this.uri}?image&cip=${cip}`;
}


update_thumbnails() {
    const me = this;
    const i = document.getElementById("thumbnails");
    if (i && this.server_project) {
        html.clear(i);
        for (const n in this.server_project.thumbnails) {
            if (this.server_project.thumbnails[n]) {
                const a = html.add_ele(i, "a");
                a.addEventListener('click', function(e) {me.select_cip_of_project(n);});
                const img = html.add_ele(a, "img");
                img.src = URL.createObjectURL(this.server_project.thumbnails[n]);
            }
        }
    }
}
*/
