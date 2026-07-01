import photogram_init, {
  InitOutput,
  WasmProject,
  WasmCip,
} from "../pkg/photogram_wasm.js";

// import { WasmMemory } from "./wasm_memory.js";
import { Tabs } from "./tabs.js";
import { Log, Logger, Severity } from "./log.js";
import { LocalStorage } from "./storage.js";
import { HtmlElement } from "./html.js";

import { FileSet } from "./file_set.js";
import { ProjectSet } from "./project_set.js";
import { Browser } from "./browser.js";
import { LensCalibrationPlot } from "./lens_calibration_plot.js";
import { ProjectEdit } from "./project_edit.js";

import { Application } from "./application.js";

enum SelectedTab {
  Help,
  Browser,
  LensCalibrationPlot,
  ProjectEdit,
  Log,
}

class TabType {
  selected_tab: SelectedTab;
  constructor(selected_tab: SelectedTab) {
    this.selected_tab = selected_tab;
  }
}

export class Photogram implements Application {
  logger: Log;
  log: Logger;

  file_set: FileSet;
  project_set: ProjectSet;

  project_locator: string | null = null;
  project: WasmProject | null = null;
  cip: WasmCip | null = null;

  // wasm_memory: WasmMemory;
  tabs: Tabs<TabType>;
  selected_tab_type: TabType | null = null;
  pending_resize: [number, number] | null;
  resize_observer: ResizeObserver;
  browser: Browser;
  lens_calibration_plot: LensCalibrationPlot;
  project_edit: ProjectEdit;

  constructor(_wasm_instance: InitOutput, _params: URLSearchParams) {
    // this.wasm_memory = new WasmMemory(wasm_instance.memory);
    this.logger = new Log("Log", Severity.Info, Severity.Warning);
    this.log = new Logger(this.logger, "main");
    const local_storage = new LocalStorage(window.localStorage, "photogram");
    this.file_set = new FileSet(local_storage, this.repopulate.bind(this));

    this.project_set = new ProjectSet(
      new Logger(this.logger, "project_set"),
      this.file_set,
      () => {},
    );

    const browser_div = new HtmlElement(document.getElementById("browser")!);
    this.browser = new Browser(
      this,
      new Logger(this.logger, "browser"),
      this.file_set,
      browser_div,
    );

    const lens_calibration_plot_div = new HtmlElement(
      document.getElementById("lens_calibration_plot")!,
    );
    this.lens_calibration_plot = new LensCalibrationPlot(
      this,
      new Logger(this.logger, "lens_calibration_plot"),
      lens_calibration_plot_div,
    );

    const project_edit_div = new HtmlElement(
      document.getElementById("project_edit")!,
    );
    this.project_edit = new ProjectEdit(
      this,
      new Logger(this.logger, "project_edit"),
      project_edit_div,
    );

    this.pending_resize = null;

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

    this.tabs = new Tabs("tab-list", this.tab_selected.bind(this), [
      ["tab-help", "Help", new TabType(SelectedTab.Help)],
      ["tab-browser", "Browser", new TabType(SelectedTab.Browser)],
      [
        "tab-lens-calibration-plot",
        "Lens Calibration",
        new TabType(SelectedTab.LensCalibrationPlot),
      ],
      [
        "tab-project-edit",
        "Project Edit",
        new TabType(SelectedTab.ProjectEdit),
      ],
      ["tab-log", "Log", new TabType(SelectedTab.Log)],
    ]);
    this.selected_tab_type = null!;
    this.tabs.select("help");

    this.file_set.get_file_list();

    this.load_project("local:nac_all_proj.json");
  }

  current_project_name(): string | null {
    return this.project_locator;
  }

  current_project(): WasmProject | null {
    return this.project;
  }

  current_cip(): WasmCip | null {
    return this.cip;
  }

  load_project(locator: string) {
    this.project_locator = locator;
    this.project = null;
    this.cip = null;
    this.project_set.load_project(
      this.project_locator,
      this.project_loaded.bind(this),
      this.project_load_error.bind(this),
    );
    this.repopulate();
  }

  project_load_error(e: string) {
    this.log.error(e);
    this.project_locator = null;
    this.repopulate();
  }

  project_loaded(project: WasmProject) {
    this.project = project;
    this.set_cip(project.cip("4V3A6042.JPG"));
  }

  set_cip(cip: WasmCip) {
    this.cip = cip;
    this.repopulate();
  }

  repopulate() {
    const project_locator = this.project_locator
      ? this.project_locator
      : "<no project open>";
    const cip_name = this.cip ? this.cip.image : "<no CIP>";
    const body_name = this.cip ? this.cip.camera.body : "<no CIP>";
    const lens_name = this.cip ? this.cip.camera.lens : "<no CIP>";
    const focal_length = this.cip
      ? this.cip.camera.focal_length.toString() + "mm"
      : "<no CIP>";
    const fovd = this.cip
      ? (
          Math.floor((Math.atan(this.cip.camera.tan_fovd) * 18000) / 3.14159) /
          100
        ).toString() + "°"
      : "<no CIP>";
    const fovh = this.cip
      ? (
          Math.floor((Math.atan(this.cip.camera.tan_fovh) * 18000) / 3.14159) /
          100
        ).toString() + "°"
      : "<no CIP>";

    HtmlElement.fold_all_of(".set-project-name", null, (a, e) => {
      e.ele.innerHTML = project_locator;
      return a;
    });
    HtmlElement.fold_all_of(".set-cip-name", null, (a, e) => {
      e.ele.innerHTML = cip_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-body", null, (a, e) => {
      e.ele.innerHTML = body_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-lens", null, (a, e) => {
      e.ele.innerHTML = lens_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-focal-length", null, (a, e) => {
      e.ele.innerHTML = focal_length;
      return a;
    });
    HtmlElement.fold_all_of(".set-fovd", null, (a, e) => {
      e.ele.innerHTML = fovd;
      return a;
    });
    HtmlElement.fold_all_of(".set-fovh", null, (a, e) => {
      e.ele.innerHTML = fovh;
      return a;
    });

    this.browser.repopulate();
    this.lens_calibration_plot.repopulate();
    this.project_edit.repopulate();
  }

  resize_canvas(e: ResizeObserverEntry[]): void {
    for (const ele of e) {
      if (ele.contentRect.width > 0 && ele.contentRect.height > 0) {
        this.pending_resize = [ele.contentRect.width, ele.contentRect.height];
        this.lens_calibration_plot.resize(this.pending_resize);
      }
    }
  }

  tab_selected(tab_type: TabType) {
    this.selected_tab_type = tab_type;

    // this.set_view_needs_update();
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
