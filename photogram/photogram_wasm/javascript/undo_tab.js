import { UndoBuffer, UndoDiv } from "./undo.js";
export class UndoTab {
    constructor(application, log, div) {
        this.application = application;
        this.undo_div = new UndoDiv(new UndoBuffer(), div);
        this.log = log;
        application.add_tab(this, null);
    }
    tab_name() {
        return "undo";
    }
    tab_text() {
        return "Undo";
    }
    tab_deselected() { }
    tab_selected() {
        this.undo_div.set_undo_buffer(this.application.current_project().get_undo_buffer());
        this.undo_div.request_fill_div();
    }
}
