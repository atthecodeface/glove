import { Table } from "./html.js";
export class ProjectEdit {
    constructor(application, log, html_div) {
        this.tab_is_selected = false;
        this.application = application;
        this.log = log;
        this.html_div = html_div;
        this.html_div.add_button("", "", this.save_project.bind(this)).add_content("Save project");
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
    tab_deselected() {
        this.tab_is_selected = false;
    }
    /** Invoked when the tab is tab_selected
     *
     * tab_project_updated will be invoked, which will repopulate the div
     */
    tab_selected() {
        this.tab_is_selected = true;
    }
    tab_project_selected(p) {
        p.add_client(this);
    }
    /** Invoked after the tab is selected or a project update occurs */
    tab_project_updated() {
        const mapped_nps = this.application.current_project().mapped_nps();
        mapped_nps.update();
        this.repopulate_nps_div();
        this.repopulate_cip_div();
    }
    tab_resize(_w, _h) {
    }
    tab_redraw() {
    }
    project_np_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_pm_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_camera_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_cip_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_mapped_nps_changed(_p) {
        if (this.tab_is_selected) {
            this.repopulate_nps_div();
            this.repopulate_cip_div();
        }
    }
    repopulate_nps_div() {
        this.nps_div.clear();
        this.nps_div.add_button("", "", this.add_new_np.bind(this)).add_content("Add named point");
        const table = new Table({ classes: "sticky_heading" });
        this.application.current_project().mapped_nps().fill_np_table(table);
        this.nps_div.add_content(table.as_html());
    }
    save_project() {
        const project = this.application.current_project();
        project.save_project(null);
    }
    add_new_np() {
        const project = this.application.current_project();
        const np_name = project.nps_get_new_name();
        project.nps_add(np_name);
    }
    repopulate_cip_div() {
        this.cip_div.clear();
        const cip_name = this.application.current_project().get_cip().cip_name;
        this.cip_div.add_ele("h2").add_content(`Current CIP '${cip_name}'`);
        const table = new Table({ classes: "sticky_heading" });
        this.application.current_project().mapped_nps().fill_pms_table(table);
        this.cip_div.add_content(table.as_html());
    }
}
