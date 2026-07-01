import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application } from "./application.js";

export class StarCalibration {
  application: Application;
  log: Logger;
  html_div: HtmlElement;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.repopulate();
  }

  repopulate() {}
}
