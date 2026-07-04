import { WasmCip, WasmNamedPoint, WasmProject } from "../pkg/photogram_wasm";

import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application } from "./application.js";
import { Cip } from "./cip.js";
import { ProjectSet } from "./project_set.js";

export class Project {
  private application: Application;
  private project_set: ProjectSet;
  private log: Logger;
  locator: string | null = null;
  private wasm_project: WasmProject | null = null;
  private modified: boolean = false;
  private promise_epoch: number = 0;
  private cip: Cip;

  /** Thumbnails loaded, one per CIP of the current project */
  private thumbnails: Map<string, HTMLImageElement>;
  private thumbnail_width: number = 256;

  /** Undo list */

  constructor(
    application: Application,
    log: Logger,
    project_set: ProjectSet,
    cip: Cip,
  ) {
    this.application = application;
    this.log = log;
    this.project_set = project_set;
    this.cip = cip;
    this.thumbnails = new Map();
  }

  nps_add(name: string): boolean {
    const current_np = this.wasm_project!.nps.get_pt(name);
    if (current_np !== undefined) {
      return false;
    }
    const np = new WasmNamedPoint(name, "yellow");
    this.wasm_project!.nps.add_pt(np);
    return true;
  }

  get_cip(): Cip {
    return this.cip;
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
    return this.modified;
  }

  get_wasm_cip(): WasmCip | null {
    return this.cip.wasm_cip;
  }

  cancel_all_promises() {
    this.promise_epoch += 1;
  }

  load_project(locator: string) {
    this.cancel_all_promises();
    this.cip.set_cip("", null);
    this.locator = locator;
    this.modified = false;
    this.thumbnails.clear();
    this.project_set.load_project(
      this.locator,
      this.project_loaded.bind(this),
      this.project_load_error.bind(this),
    );
  }

  project_loaded(wasm_project: WasmProject): void {
    this.wasm_project = wasm_project;
    this.application.project_load_completed(true);
    this.log.info(`Project ${this.locator} loaded`);

    this.thumbnails = new Map();
    for (let i = 0; i < this.wasm_project.ncips(); i++) {
      const cip_name = this.wasm_project.cip_name(i)!;
      const promise = this.project_set.promise_fetch_thumbnail(
        this.locator!,
        cip_name,
        this.thumbnail_width,
      );
      if (promise !== null) {
        promise
          .then((jpg) => {
            this.thumbnail_loaded(this.promise_epoch, cip_name, jpg);
          })
          .catch(this.log_exception.bind(this));
      }
    }
  }

  log_exception(e: Error) {
    this.log.error(e.message);
  }

  thumbnail_loaded(epoch: number, cip_name: string, jpg: Blob) {
    if (epoch != this.promise_epoch) {
      return;
    }
    this.log.info(`Thumbnail ${cip_name} loaded for project ${this.locator}`);
    const img = new Image();
    img.src = URL.createObjectURL(jpg);
    this.thumbnails.set(cip_name, img);
    this.application.thumbnails_updated();
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
    this.log.info(`Project ${this.locator} saved`);
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

  cip_image_loaded(epoch: number, cip_name: string, jpg: Blob) {
    if (epoch == this.promise_epoch) {
      this.log.info(`Image ${cip_name} loaded for project ${this.locator}`);
      this.cip.set_cip_image_data(cip_name, jpg);
    }
  }

  set_cip(cip_name: string) {
    const wasm_cip = this.get_cip_by_name(cip_name);
    this.cip.set_cip(cip_name, wasm_cip);
    if (this.locator !== null) {
      const promise = this.project_set.promise_fetch_image(
        this.locator,
        cip_name,
      );
      if (promise !== null)
        promise
          .then((jpg) => {
            this.cip_image_loaded(this.promise_epoch, cip_name, jpg);
          })
          .catch(this.log_exception.bind(this));
    }
  }
}
