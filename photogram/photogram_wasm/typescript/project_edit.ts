import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application } from "./application.js";

export class ProjectEdit {
  application: Application;
  log: Logger;
  html_div: HtmlElement;
  ctl_div: HtmlElement;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.ctl_div = this.html_div.add_ele("div");
  }

  repopulate() {}
}
