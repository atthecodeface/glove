import { HtmlElement, Table } from "./html.js";
import { Logger } from "./log.js";

import { Application, ApplicationTab } from "./application.js";
import { Project, ProjectClient } from "./project.js";

export class ProjectEdit implements ProjectClient, ApplicationTab {
  application: Application;
  log: Logger;
  html_div: HtmlElement;
  nps_div: HtmlElement;
  cip_div: HtmlElement;

  tab_is_selected: boolean = false;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.html_div.add_button("", "", this.save_project.bind(this)).add_content("Save project");
    this.nps_div = this.html_div.add_ele("div", { id: "project_edit_nps" });
    this.cip_div = this.html_div.add_ele("div", { id: "project_edit_cip" });

    application.add_tab(this, null);
  }

  tab_name(): string {
    return "project-edit";
  }

  tab_text(): string {
    return "Project Edit";
  }

  tab_deselected(): void {
    this.tab_is_selected = false;
  }

  /** Invoked when the tab is tab_selected
   *
   * tab_project_updated will be invoked, which will repopulate the div
   */
  tab_selected(): void {
    this.tab_is_selected = true;
  }

  tab_project_selected(p: Project): void {
    p.add_client(this);
  }

  /** Invoked after the tab is selected or a project update occurs */
  tab_project_updated(): void {
    const mapped_nps = this.application.current_project().mapped_nps();
    mapped_nps.update();

    this.repopulate_nps_div();
    this.repopulate_cip_div();
  }

  tab_resize(_w:number, _h:number): void {
  }

  tab_redraw(): void {
  }

  project_np_changed(_p: Project): void {
    if (this.tab_is_selected) { this.application.set_project_updated(); }
  }

  project_pm_changed(_p: Project): void {
    if (this.tab_is_selected) { this.application.set_project_updated(); }
  }

  project_camera_changed(_p: Project): void {
    if (this.tab_is_selected) { this.application.set_project_updated(); }
  }

  project_cip_changed(_p: Project): void {
    if (this.tab_is_selected) { this.application.set_project_updated(); }
  }

  project_mapped_nps_changed(_p: Project): void {
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
    let i = 0;
    let np_name = "";
    const project = this.application.current_project();
    const wasm_nps = project.get_wasm_nps()!;
    while (true) {
      np_name = `np_${i}`;
      if (wasm_nps.get_pt(np_name) === undefined) {
        break;
      }
      i += 1;
    }
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
