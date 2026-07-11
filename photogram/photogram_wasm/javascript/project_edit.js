import { Table } from "./html.js";
import { MappedNps } from "./mapped_nps.js";
export class ProjectEdit {
    constructor(application, log, html_div) {
        this.mapped_nps = null;
        this.application = application;
        this.log = log;
        this.html_div = html_div;
        this.nps_div = this.html_div.add_ele("div", { id: "project_edit_nps" });
        this.cip_div = this.html_div.add_ele("div", { id: "project_edit_cip" });
        application.add_tab(this, null);
    }
    tab_name() {
        return "project-edit";
    }
    tab_text() {
        return "Project Edit";
    }
    tab_deselected() { }
    tab_selected() {
        this.repopulate();
    }
    repopulate() {
        const project = this.application.current_project();
        if (project.get_wasm_nps() === null) {
            this.mapped_nps = null;
        }
        else {
            this.mapped_nps = new MappedNps(project);
            this.mapped_nps.map_with_cip(project.get_cip());
        }
        this.repopulate_nps_div();
        this.repopulate_cip_div();
    }
    repopulate_nps_div() {
        this.nps_div.clear();
        if (this.mapped_nps === null) {
            return;
        }
        const table = new Table({ classes: "sticky_heading" });
        this.mapped_nps.fill_np_table(table);
        this.nps_div.add_content(table.as_html());
    }
    repopulate_cip_div() {
        this.cip_div.clear();
        if (this.mapped_nps === null) {
            return;
        }
        const cip_name = this.application.current_project().get_cip().cip_name;
        this.cip_div.add_ele("h2").add_content(`Current CIP '${cip_name}'`);
        const table = new Table({ classes: "sticky_heading" });
        this.mapped_nps.fill_pms_table(table);
        this.cip_div.add_content(table.as_html());
    }
}
