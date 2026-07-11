import { HtmlElement } from "./html.js";

export interface UndoableAction<T> {
  fwd(t: T): void;
  rev(t: T): void;
  fwd_text(): string;
  rev_text(): string;
}

/** UndoBuffer could be a tree like git, with branches which have been popped back from (by undo) and then developed further
 *
 * The simplest is a (limited size?) buffer of actions, and 'future actions'
 * which is pushed onto on 'undo', which is then cleared on a new action
 * (keeping it linear)
 */
export class UndoBuffer<T> {
  undo_actions: UndoableAction<T>[] = [];
  redo_actions: UndoableAction<T>[] = [];

  do_action(action: UndoableAction<T>): void {
    this.undo_actions.push(action);
    this.redo_actions = [];
  }
  undo(): UndoableAction<T> | null {
    const action = this.undo_actions.pop();
    if (action === undefined) {
      return null;
    }
    this.redo_actions.push(action);
    return action;
  }
  redo(): UndoableAction<T> | null {
    const action = this.redo_actions.pop();
    if (action === undefined) {
      return null;
    }
    this.undo_actions.push(action);
    return action;
  }
}

export class UndoDiv<T> {
  undo_buffer: UndoBuffer<T>;
  div: HtmlElement;
  refill_pending: boolean;

  /** Create a new Log that will fill the given 'div' which has an 'id' of div_id
   *
   * @param {HtmlElement | string}  div an HtmlElement, or 'id' of a div in the document, to place the log into; if none is provided then logging is only to the console
   *
   * @param {Severity} min_severity Minimum severity for logging in the div; defaults to Info
   *
   * @param {Severity} console_min_severity Minimum severity for logging in the console; defaults to Warning
   */
  constructor(undo_buffer: UndoBuffer<T>, div: HtmlElement) {
    this.undo_buffer = undo_buffer;
    this.div = div;
    this.refill_pending = false;
  }

  set_undo_buffer(undo_buffer:UndoBuffer<T>) {
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
      table.add_body([table.add_span(e.fwd_text())]);
    }
    const actions = this.undo_buffer.redo_actions;
    for (let i = actions.length - 1; i>=0; i--) {
      table.add_body([table.add_span(actions[i]!.rev_text())]);
    }
    table.as_html();
  }
}
