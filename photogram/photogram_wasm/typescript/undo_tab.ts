import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";

import { Application, ApplicationTab } from "./application.js";
import { UndoBuffer, UndoDiv } from "./undo.js";
import { Project } from "./project.js";
export class UndoTab implements ApplicationTab {
  application: Application;
  log: Logger;
  undo_div: UndoDiv<Project>;

  constructor(application: Application, log: Logger, div: HtmlElement) {
    this.application = application;
    this.undo_div = new UndoDiv(new UndoBuffer(), div);
    this.log = log;
    application.add_tab(this, null);
  }

  tab_name(): string {
    return "undo";
  }
  tab_text(): string {
    return "Undo";
  }
  tab_deselected(): void { }
  tab_selected(): void {
    this.undo_div.set_undo_buffer(this.application.current_project().get_undo_buffer());
    this.undo_div.request_fill_div();
  }
}
