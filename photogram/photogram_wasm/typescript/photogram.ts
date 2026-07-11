import photogram_init, { InitOutput } from "../pkg/photogram_wasm.js";

import { WasmMemory } from "./wasm_memory.js";
import { Tabs } from "./tabs.js";
import { Log, Logger, Severity } from "./log.js";
import { LocalStorage } from "./storage.js";
import { HtmlElement } from "./html.js";

import { Cip } from "./cip.js";
import { FileSet } from "./file_set.js";
import { Project } from "./project.js";
import { ProjectSet } from "./project_set.js";
import { WebglCanvas, WebglCanvasClient } from "./webgl_canvas.js";
import { Browser } from "./browser.js";
import { LensCalibrationPlot } from "./lens_calibration_plot.js";
import { StarCalibration } from "./star_calibration.js";
import { ProjectEdit } from "./project_edit.js";

import { Application, ApplicationTab } from "./application.js";
import { UndoTab } from "./undo_tab.js";

class TabType {
  web_canvas_client: WebglCanvasClient | null = null;
  application_tab: ApplicationTab | null = null;
  constructor() {}
  set_application_tab(application_tab: ApplicationTab | null) {
    this.application_tab = application_tab;
  }
  select(): void {
    if (this.application_tab !== null) {
      this.application_tab.tab_selected();
    }
  }
  deselect(): void {
    if (this.application_tab !== null) {
      this.application_tab.tab_deselected();
    }
  }
  set_client(web_canvas_view: WebglCanvasClient): TabType {
    this.web_canvas_client = web_canvas_view;
    return this;
  }
}

export class Photogram implements Application {
  wasm_memory: WasmMemory;
  app_logger: Log;
  log: Logger;

  file_set: FileSet;
  project_set: ProjectSet;

  project: Project;
  cip: Cip;

  // wasm_memory: WasmMemory;
  tabs: Tabs<TabType>;
  selected_tab_type: TabType | null = null;
  resizable_size: [number, number] = [50, 50];
  pending_resize: boolean = false;
  view_needs_update: boolean = false;
  resize_observer: ResizeObserver;

  webgl_canvas: WebglCanvas;
  browser: Browser;
  lens_calibration_plot: LensCalibrationPlot;
  star_calibration: StarCalibration;
  project_edit: ProjectEdit;
  undo: UndoTab;

  constructor(wasm_instance: InitOutput, _params: URLSearchParams) {
    this.wasm_memory = new WasmMemory(wasm_instance.memory);
    this.app_logger = new Log("Log", Severity.Info, Severity.Warning);
    this.log = new Logger(this.app_logger, "main");
    const local_storage = new LocalStorage(window.localStorage, "photogram");
    this.file_set = new FileSet(local_storage, this.repopulate.bind(this));
    this.cip = new Cip(new Logger(this.app_logger, "cip"));

    this.project_set = new ProjectSet(
      new Logger(this.app_logger, "project_set"),
      this.file_set,
      () => {},
    );
    this.project = new Project(
      this,
      new Logger(this.app_logger, "project"),
      this.project_set,
      this.cip,
    );

    const webgl_canvas = new HtmlElement(
      document.getElementById("webgl-canvas")!,
    );
    this.webgl_canvas = new WebglCanvas(
      this,
      new Logger(this.app_logger, "webgl"),
      webgl_canvas,
    );

    this.tabs = new Tabs("tab-list", this.tab_selected.bind(this), [
      ["tab-help", "Help", new TabType()],
    ]);

    const browser_div = new HtmlElement(document.getElementById("browser")!);
    this.browser = new Browser(
      this,
      new Logger(this.app_logger, "browser"),
      this.file_set,
      browser_div,
    );

    const star_calibration_div = new HtmlElement(
      document.getElementById("star_calibration")!,
    );
    this.star_calibration = new StarCalibration(
      this,
      new Logger(this.app_logger, "star_calibration"),
      star_calibration_div,
    );

    const lens_calibration_plot_div = new HtmlElement(
      document.getElementById("lens_calibration_plot")!,
    );
    this.lens_calibration_plot = new LensCalibrationPlot(
      this,
      new Logger(this.app_logger, "lens_calibration_plot"),
      lens_calibration_plot_div,
    );

    const project_edit_div = new HtmlElement(
      document.getElementById("project_edit")!,
    );
    this.project_edit = new ProjectEdit(
      this,
      new Logger(this.app_logger, "project_edit"),
      project_edit_div,
    );

    const undo_div = new HtmlElement(
      document.getElementById("undo")!,
    );
    this.undo = new UndoTab (
      this,
      new Logger(this.app_logger, "undo"),
      undo_div,
    );

    this.pending_resize = false;

    this.resize_observer = new ResizeObserver(this.resize_canvas.bind(this));
    for (const resizable_content of document.getElementsByClassName(
      "get_size_of_this",
    )) {
      this.resize_observer.observe(resizable_content);
    }
    const tab_list = document.getElementById("tab-list")!;
    const tab_list_height = tab_list.offsetHeight;
    HtmlElement.fold_all_of(".set-to-size-of-tab-list", null, (a, e) => {
      e.ele.style.height = `${tab_list_height}px`;
      return a;
    });

    this.file_set.get_file_list();

    this.tabs.add_tab("tab-log", "Log", new TabType());

    this.selected_tab_type = null!;
    this.tabs.select("help");

    // this.load_project("local:nac_all_proj.json");
    // this.load_project("server:nac_all_proj");
    this.load_project("server:lens_calibrations_proj");
  }

  logger(): Log {
    return this.app_logger;
  }

  add_tab(
    application_tab: ApplicationTab,
    web_canvas_client: WebglCanvasClient | null,
  ): void {
    const tab_type = new TabType();
    tab_type.set_application_tab(application_tab);
    if (web_canvas_client !== null) {
      tab_type.set_client(web_canvas_client);
      this.webgl_canvas.create(web_canvas_client);
    }
    this.tabs.add_tab(
      "tab-" + application_tab.tab_name(),
      application_tab.tab_text(),
      tab_type,
    );
  }

  get_resizable_content_size(): [number, number] {
    return this.resizable_size;
  }

  current_project_name(): string | null {
    return this.project.locator;
  }

  current_project(): Project {
    return this.project;
  }

  load_project(locator: string) {
    this.project.load_project(locator);
    this.cip.set_cip("", null);
  }

  project_load_completed(success: boolean): void {
    if (success) {
      const cip_name = this.project.get_cip_name(0)!;
      this.set_cip(cip_name);
    } else {
    }
    this.repopulate();
  }

  project_save_completed(success: boolean): void {
    // Note should backup server to local on save?
    if (success) {
      this.log.info("Saved project");
    } else {
    }
  }

  set_cip(cip_name: string) {
    this.project.set_cip(cip_name);
    this.repopulate();
  }

  repopulate() {
    this.project.repopulate();
    this.cip.repopulate();
  }

  thumbnails_updated() {}

  resize_canvas(e: ResizeObserverEntry[]): void {
    for (const ele of e) {
      if (ele.contentRect.width > 0 && ele.contentRect.height > 0) {
        this.pending_resize = true;
        this.resizable_size = [ele.contentRect.width, ele.contentRect.height];
        this.set_view_needs_update();
      }
    }
  }

  tab_selected(tab_type: TabType) {
    if (this.selected_tab_type !== tab_type) {
      if (this.selected_tab_type !== null) {
        this.selected_tab_type.deselect();
      }
    }
    this.selected_tab_type = tab_type;

    if (this.selected_tab_type.web_canvas_client === null) {
      this.webgl_canvas.canvas.hidden = true;
    } else {
      this.webgl_canvas.canvas.hidden = false;
      this.webgl_canvas.mouse.set_client(
        this.selected_tab_type.web_canvas_client,
      );
      this.selected_tab_type.web_canvas_client.webgl_resize(
        this.resizable_size[0],
        this.resizable_size[1],
      );
    }
    this.selected_tab_type.select();
    this.set_view_needs_update();
  }

  /** Mark the view as needing an update
   *
   * This is lightweight as it is used in animation
   */
  set_view_needs_update() {
    if (!this.view_needs_update) {
      this.view_needs_update = true;
      requestAnimationFrame(this.update_view.bind(this));
    }
  }

  /** Update the view, because of a view change, time change, animation step, etc
   *
   * This is lightweight as it is used in animation
   */
  update_view() {
    if (this.selected_tab_type === null) {
      return;
    }
    if (this.pending_resize) {
      const w = this.resizable_size[0];
      const h = this.resizable_size[1];
      this.lens_calibration_plot.resize(this.resizable_size);
      HtmlElement.fold_all_of(".set-size-of-this", null, (a, e) => {
        (e.ele as any).width = w;
        (e.ele as any).height = h;
        return a;
      });
      if (this.selected_tab_type.web_canvas_client !== null) {
        this.selected_tab_type.web_canvas_client.webgl_resize(
          this.resizable_size[0],
          this.resizable_size[1],
        );
      }

      //      this.vp.set_resizable_content_size(this.pending_resize);
      this.pending_resize = false;
      this.view_needs_update = true;
    }
    if (!this.view_needs_update) {
      return;
    }

    // this.controls.update();

    if (this.selected_tab_type.web_canvas_client !== null) {
      this.webgl_canvas.redraw(this.selected_tab_type.web_canvas_client);
    }

    this.view_needs_update = false;
  }
}

//a Top level on load...
(window as any).photogram = null;
function complete_init(photogram_wasm: InitOutput) {
  (window as any).photogram = new Photogram(
    photogram_wasm,
    new URLSearchParams(window.location.search),
  );
}

window.addEventListener("load", (_e) => {
  photogram_init().then((x) => {
    complete_init(x);
  });
});
