import photogram_init from "../pkg/photogram_wasm.js";
import { WasmMemory } from "./wasm_memory.js";
import { Tabs } from "./tabs.js";
import { Log, Logger, Severity } from "./log.js";
import { LocalStorage } from "./storage.js";
import { HtmlElement } from "./html.js";
import { Cip } from "./cip.js";
import { FileSet } from "./file_set.js";
import { Project } from "./project.js";
import { ProjectSet } from "./project_set.js";
import { WebglCanvas } from "./webgl_canvas.js";
import { Browser } from "./browser.js";
import { LensCalibrationPlot } from "./lens_calibration_plot.js";
import { StarCalibration } from "./star_calibration.js";
import { ProjectEdit } from "./project_edit.js";
var SelectedTab;
(function (SelectedTab) {
    SelectedTab[SelectedTab["Help"] = 0] = "Help";
    SelectedTab[SelectedTab["Browser"] = 1] = "Browser";
    SelectedTab[SelectedTab["StarCalibration"] = 2] = "StarCalibration";
    SelectedTab[SelectedTab["LensCalibrationPlot"] = 3] = "LensCalibrationPlot";
    SelectedTab[SelectedTab["ProjectEdit"] = 4] = "ProjectEdit";
    SelectedTab[SelectedTab["Log"] = 5] = "Log";
})(SelectedTab || (SelectedTab = {}));
class TabType {
    constructor(selected_tab) {
        this.web_canvas_client = null;
        this.selected_tab = selected_tab;
    }
    set_client(web_canvas_view) {
        this.web_canvas_client = web_canvas_view;
        return this;
    }
}
export class Photogram {
    constructor(wasm_instance, _params) {
        this.selected_tab_type = null;
        this.resizable_size = [50, 50];
        this.pending_resize = false;
        this.view_needs_update = false;
        this.wasm_memory = new WasmMemory(wasm_instance.memory);
        this.app_logger = new Log("Log", Severity.Info, Severity.Warning);
        this.log = new Logger(this.app_logger, "main");
        const local_storage = new LocalStorage(window.localStorage, "photogram");
        this.file_set = new FileSet(local_storage, this.repopulate.bind(this));
        this.cip = new Cip(new Logger(this.app_logger, "cip"));
        this.project_set = new ProjectSet(new Logger(this.app_logger, "project_set"), this.file_set, () => { });
        this.project = new Project(this, new Logger(this.app_logger, "project"), this.project_set, this.cip);
        const webgl_canvas = new HtmlElement(document.getElementById("webgl-canvas"));
        this.webgl_canvas = new WebglCanvas(this, new Logger(this.app_logger, "webgl"), webgl_canvas);
        const browser_div = new HtmlElement(document.getElementById("browser"));
        this.browser = new Browser(this, new Logger(this.app_logger, "browser"), this.file_set, browser_div);
        const star_calibration_div = new HtmlElement(document.getElementById("star_calibration"));
        this.star_calibration = new StarCalibration(this, new Logger(this.app_logger, "star_calibrationt"), star_calibration_div);
        const lens_calibration_plot_div = new HtmlElement(document.getElementById("lens_calibration_plot"));
        this.lens_calibration_plot = new LensCalibrationPlot(this, new Logger(this.app_logger, "lens_calibration_plot"), lens_calibration_plot_div);
        const project_edit_div = new HtmlElement(document.getElementById("project_edit"));
        this.project_edit = new ProjectEdit(this, new Logger(this.app_logger, "project_edit"), project_edit_div);
        this.pending_resize = false;
        this.resize_observer = new ResizeObserver(this.resize_canvas.bind(this));
        for (const resizable_content of document.getElementsByClassName("get_size_of_this")) {
            this.resize_observer.observe(resizable_content);
        }
        const tab_list = document.getElementById("tab-list");
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
                "tab-star-calibration",
                "Star Calibration",
                new TabType(SelectedTab.StarCalibration).set_client(this.star_calibration),
            ],
            [
                "tab-project-edit",
                "Project Edit",
                new TabType(SelectedTab.ProjectEdit),
            ],
            ["tab-log", "Log", new TabType(SelectedTab.Log)],
        ]);
        this.selected_tab_type = null;
        this.tabs.select("help");
        this.file_set.get_file_list();
        for (const t of this.tabs.tabs) {
            if (t.client.web_canvas_client !== null) {
                this.webgl_canvas.create(t.client.web_canvas_client);
            }
        }
        // this.load_project("local:nac_all_proj.json");
        // this.load_project("server:nac_all_proj");
        this.load_project("server:lens_calibrations_proj");
    }
    logger() {
        return this.app_logger;
    }
    get_resizable_content_size() {
        return this.resizable_size;
    }
    current_project_name() {
        return this.project.locator;
    }
    current_project() {
        return this.project;
    }
    load_project(locator) {
        this.project.load_project(locator);
        this.cip.set_cip("", null);
    }
    project_load_completed(success) {
        if (success) {
            const cip_name = this.project.get_cip_name(0);
            this.set_cip(cip_name);
        }
        else {
        }
        this.repopulate();
    }
    project_save_completed(success) {
        // Note should backup server to local on save?
        if (success) {
            this.log.info("Saved project");
        }
        else {
        }
    }
    set_cip(cip_name) {
        this.project.set_cip(cip_name);
        this.repopulate();
    }
    repopulate() {
        this.project.repopulate();
        this.cip.repopulate();
        this.browser.repopulate();
        this.lens_calibration_plot.repopulate();
        this.project_edit.repopulate();
    }
    thumbnails_updated() { }
    resize_canvas(e) {
        for (const ele of e) {
            if (ele.contentRect.width > 0 && ele.contentRect.height > 0) {
                this.pending_resize = true;
                this.resizable_size = [ele.contentRect.width, ele.contentRect.height];
                this.set_view_needs_update();
            }
        }
    }
    tab_selected(tab_type) {
        this.selected_tab_type = tab_type;
        if (this.selected_tab_type.web_canvas_client === null) {
            this.webgl_canvas.canvas.hidden = true;
        }
        else {
            this.webgl_canvas.canvas.hidden = false;
            this.webgl_canvas.mouse.set_client(this.selected_tab_type.web_canvas_client);
            this.selected_tab_type.web_canvas_client.webgl_resize(this.resizable_size[0], this.resizable_size[1]);
        }
        this.set_view_needs_update();
    }
    /// Mark the view as needing an update
    set_view_needs_update() {
        if (!this.view_needs_update) {
            this.view_needs_update = true;
            requestAnimationFrame(this.update_view.bind(this));
        }
    }
    /// Update the view, because of a view change, time change, etc
    update_view() {
        if (this.selected_tab_type === null) {
            return;
        }
        if (this.pending_resize) {
            const w = this.resizable_size[0];
            const h = this.resizable_size[1];
            this.lens_calibration_plot.resize(this.resizable_size);
            HtmlElement.fold_all_of(".set-size-of-this", null, (a, e) => {
                e.ele.width = w;
                e.ele.height = h;
                return a;
            });
            if (this.selected_tab_type.web_canvas_client !== null) {
                this.selected_tab_type.web_canvas_client.webgl_resize(this.resizable_size[0], this.resizable_size[1]);
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
window.photogram = null;
function complete_init(photogram_wasm) {
    window.photogram = new Photogram(photogram_wasm, new URLSearchParams(window.location.search));
}
window.addEventListener("load", (_e) => {
    photogram_init().then((x) => {
        complete_init(x);
    });
});
