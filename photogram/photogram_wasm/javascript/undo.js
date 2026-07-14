/** UndoBuffer could be a tree like git, with branches which have been popped back from (by undo) and then developed further
 *
 * The simplest is a (limited size?) buffer of actions, and 'future actions'
 * which is pushed onto on 'undo', which is then cleared on a new action
 * (keeping it linear)
 */
export class UndoBuffer {
    constructor() {
        this.undo_actions = [];
        this.redo_actions = [];
    }
    do_action(action) {
        this.undo_actions.push(action);
        this.redo_actions = [];
    }
    undo() {
        const action = this.undo_actions.pop();
        if (action === undefined) {
            return null;
        }
        this.redo_actions.push(action);
        return action;
    }
    redo() {
        const action = this.redo_actions.pop();
        if (action === undefined) {
            return null;
        }
        this.undo_actions.push(action);
        return action;
    }
}
export class UndoDiv {
    /** Create a new Log that will fill the given 'div' which has an 'id' of div_id
     *
     * @param {HtmlElement | string}  div an HtmlElement, or 'id' of a div in the document, to place the log into; if none is provided then logging is only to the console
     *
     * @param {Severity} min_severity Minimum severity for logging in the div; defaults to Info
     *
     * @param {Severity} console_min_severity Minimum severity for logging in the console; defaults to Warning
     */
    constructor(undo_buffer, div) {
        this.undo_buffer = undo_buffer;
        this.div = div;
        this.refill_pending = false;
    }
    set_undo_buffer(undo_buffer) {
        this.undo_buffer = undo_buffer;
    }
    request_fill_div() {
        if (!this.refill_pending) {
            requestAnimationFrame((_time) => this.fill_div());
        }
        this.refill_pending = true;
    }
    fill_div() {
        this.refill_pending = false;
        this.div.clear();
        const table = this.div.add_table({ id: "undo_table" });
        table.add_headings(["Action"]);
        for (const e of this.undo_buffer.undo_actions) {
            const text = e.fwd_text();
            const d = table.add_ele("div");
            let first = true;
            for (const t of text) {
                if (!first) {
                    d.add_ele("br");
                }
                d.add_span(t);
                first = false;
            }
            table.add_body([d]);
        }
        const actions = this.undo_buffer.redo_actions;
        for (let i = actions.length - 1; i >= 0; i--) {
            const text = actions[i].rev_text();
            const d = table.add_ele("div");
            let first = true;
            for (const t of text) {
                if (!first) {
                    d.add_ele("br");
                }
                d.add_span(t);
                first = false;
            }
            table.add_body([d]);
        }
        table.as_html();
    }
}
