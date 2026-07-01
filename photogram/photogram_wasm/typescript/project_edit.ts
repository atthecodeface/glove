import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application } from "./application.js";

export class ProjectEdit {
  application: Application;
  log: Logger;
  html_div: HtmlElement;
  info_div: HtmlElement;
  ctl_div: HtmlElement;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.info_div = this.html_div.add_ele("div");
    this.ctl_div = this.html_div.add_ele("div");
    this.repopulate();
  }

  repopulate() {
    this.populate_info();
  }

  populate_info() {
    this.info_div.clear();
    this.info_div.add_label().add_content("Current project:");
    const name = this.application.current_project_name();
    if (name !== null) {
      this.info_div.add_span(name);
    }

    const project = this.application.current_project();
    if (project !== null) {
      this.info_div.add_label().add_content("CIP 0: " + project.cip_name(0));
      this.info_div
        .add_label()
        .add_content(` Num nps: ${project.nps.num_points()}`);
    }
  }
}
