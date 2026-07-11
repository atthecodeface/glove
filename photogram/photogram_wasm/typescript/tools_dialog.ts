import { HtmlElement } from "./html.js";
import { Tabs } from "./tabs.js";

export interface ToolsDialogClient<T> {
  tools_dialog_add_tabs(tools_dialog: ToolsDialog<T>, tabs: Tabs<T>): void;
  tools_dialog_tab_selected(t: T, id: string): void;
}
export class ToolsDialog<T> {
  client: ToolsDialogClient<T>;
  div: HtmlElement;
  dialog: HtmlElement;
  tabs: Tabs<T>;
  open_timeout: number = 0;

  constructor(
    client: ToolsDialogClient<T>,
    parent: HtmlElement,
    open_timeout: number = 0,
  ) {
    this.client = client;
    this.div = parent.add_ele("div", {
      classes: "movable-dialog-parent",
    });
    this.dialog = this.div.add_dialog(
      true,
      this.populate_dialog.bind(this),
      open_timeout,
      {
        classes: "movable-dialog",
      },
    );
    this.tabs = new Tabs(
      this.dialog.add_ele("div", { classes: "tab-list" }),
      this.tab_select.bind(this),
      [],
    );
    this.client.tools_dialog_add_tabs(this, this.tabs);
  }

  add_tab_div(div_id: string, classes: string = ""): HtmlElement {
    return this.dialog.add_ele("div", { id: div_id, classes: classes });
  }

  tab_select(t: T, id: string) {
    this.client.tools_dialog_tab_selected(t, id);
  }
  open_dialog() {
    (this.dialog.ele as HTMLDialogElement).show();
  }

  populate_dialog(_dialog: HtmlElement): void {}
}
