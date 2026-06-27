import photogram_init, { InitOutput } from "../pkg/photogram_wasm.js";

// import { WasmMemory } from "./wasm_memory.js";
import { Tabs } from "./tabs.js";
import { Log, Logger, Severity } from "./log.js";
import { Browser } from "./browser.js";
import { LocalStorage } from "./storage.js";
import { HtmlElement } from "./html.js";

enum SelectedTab {
  Help,
  Browser,
  Log,
}

class TabType {
  selected_tab: SelectedTab;
  constructor(selected_tab: SelectedTab) {
    this.selected_tab = selected_tab;
  }
}

export class Photogram {
  logger: Log;
  log: Logger;
  // wasm_memory: WasmMemory;
  tabs: Tabs<TabType>;
  selected_tab_type: TabType | null = null;
  pending_resize: [number, number] | null;
  resize_observer: ResizeObserver;
  browser: Browser;

  constructor(_wasm_instance: InitOutput, _params: URLSearchParams) {
    // this.wasm_memory = new WasmMemory(wasm_instance.memory);
    this.logger = new Log("Log", Severity.Info, Severity.Warning);
    this.log = new Logger(this.logger, "main");
    const local_storage = new LocalStorage(window.localStorage, "photogram");
    const browser_div = new HtmlElement(document.getElementById("browser")!);
    this.browser = new Browser(
      local_storage,
      browser_div,
      new Logger(this.logger, "browser"),
    );

    this.pending_resize = null;

    this.resize_observer = new ResizeObserver(this.resize_canvas.bind(this));
    for (const resizable_content of document.getElementsByClassName(
      "get_size_of_this",
    )) {
      this.resize_observer.observe(resizable_content);
    }

    this.tabs = new Tabs("tab-list", this.tab_selected.bind(this), [
      ["tab-help", "Help", new TabType(SelectedTab.Help)],
      ["tab-browser", "Browser", new TabType(SelectedTab.Browser)],
      ["tab-log", "Log", new TabType(SelectedTab.Log)],
    ]);
    this.selected_tab_type = null!;
    this.tabs.select("help");
  }

  resize_canvas(e: ResizeObserverEntry[]): void {
    for (const ele of e) {
      console.log(ele.contentRect, ele.target.id);
      if (ele.contentRect.width > 0 && ele.contentRect.height > 0) {
        this.pending_resize = [ele.contentRect.width, ele.contentRect.height];
      }
    }
  }

  tab_selected(tab_type: TabType) {
    this.selected_tab_type = tab_type;

    // this.set_view_needs_update();
  }
}

//a Top level on load...
(window as any).star_catalog = null;
function complete_init(star_catalog_wasm: InitOutput) {
  (window as any).star_catalog = new Photogram(
    star_catalog_wasm,
    new URLSearchParams(window.location.search),
  );
}

window.addEventListener("load", (_e) => {
  photogram_init().then((x) => {
    complete_init(x);
  });
});
