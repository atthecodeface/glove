import { Tabs } from "./tabs.js";
export class ToolsDialog {
    constructor(client, parent, open_timeout = 0) {
        this.open_timeout = 0;
        this.client = client;
        this.div = parent.add_ele("div", {
            classes: "movable-dialog-parent",
        });
        this.dialog = this.div.add_dialog(true, this.populate_dialog.bind(this), open_timeout, {
            classes: "movable-dialog",
        });
        this.tabs = new Tabs(this.dialog.add_ele("div", { classes: "tab-list" }), this.tab_select.bind(this), []);
        this.client.tools_dialog_add_tabs(this, this.tabs);
    }
    add_tab_div(div_id, classes = "") {
        return this.dialog.add_ele("div", { id: div_id, classes: classes });
    }
    tab_select(t, id) {
        this.client.tools_dialog_tab_selected(t, id);
    }
    open_dialog() {
        this.dialog.ele.show();
    }
    populate_dialog(_dialog) { }
}
