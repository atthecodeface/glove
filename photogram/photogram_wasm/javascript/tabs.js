/**
 * History:
 *
 * 21 May:
 *   Major rework from Tabbed to Tabs
 *
 * 13th April:
 *
 *   Converted from JavaScript
 *
 *   Requires a top level Tabs() creation from init complete
 *
 *   Tabbed.hash_change => select_hash; Tab.id => Tab.get_hash; Tab.select_tab => private
 */
import { HtmlElement } from "./html.js";
/**
 * A 'tab' in a tabbed page; this is private to the library
 */
class Tab {
    /**
     * Create a new Tab given its parent, 'li' element and tab number
     *
     * The 'li' has at least one 'a' element in it, with an 'href' of '#tab-<id>',
     * where 'tab-<id>' is the *id* of the div that contains the contents of that
     * tab
     */
    constructor(li, div, name, client) {
        this.li = li;
        this.div = div;
        this.name = name;
        this.client = client;
        this.set_hidden(true);
    }
    /**
     * Set the 'hidden' style for the div associated with this tab
     */
    set_hidden(hidden) {
        this.div.ele.hidden = hidden;
        if (hidden) {
            this.li.ele.className = "";
        }
        else {
            this.li.ele.className = "active";
        }
    }
}
/**
 * A class that handles a set of Tabs, only one of which should be selected, and that will become 'unhidden' while the others are 'hidden'
 */
export class Tabs {
    /**
     * Create a new set of tabs whose tab list can be selected with 'container_select'
     *
     * This tab list must be an element that contains a 'ul' element, which in
     * turn has 'li' for each tab, with each 'li' having an 'a' with an 'href'
     * identifying the tab it is associated with.
     */
    constructor(div_id, tab_select_callback, tabs) {
        this.tabs = [];
        this.callback = tab_select_callback;
        this.selected_tab = null;
        const tab_list = document.getElementById(div_id);
        if (tab_list === null) {
            throw new Error(`Failed to make 'Tabs' as ${div_id} was not found`);
        }
        this.ul = new HtmlElement(tab_list).clear().add_ele("ul");
        for (const [name, content, client] of tabs) {
            this.add_tab(name, content, client);
        }
        window.addEventListener("hashchange", () => {
            this.select(location.hash.slice(1));
        });
    }
    add_tab(name, content, client) {
        const href = "#" + name;
        const div = document.querySelector(href);
        if (div === null || !(div instanceof HTMLDivElement)) {
            throw new Error(`tab "${href}" does not have a relevant div in the document`);
        }
        const li = this.ul.add_ele("li");
        const a = li.add_ele("a", {}, [["href", href]]);
        a.add_content(content);
        const tab = new Tab(li, new HtmlElement(div), name, client);
        this.tabs.push(tab);
        a.ele.addEventListener("click", (e) => {
            this.select_tab(tab);
            e.preventDefault();
        });
    }
    tab(name) {
        for (const tab of this.tabs) {
            if (name === tab.name) {
                return tab.client;
            }
        }
        return null;
    }
    /** Select the tab of the given name 'name'
     *
     * This will invoke the 'tab_select_callback' provided by the client with the
     * selected tab's '#<tab-name>', if the tab number changes
     *
     *  Invoked when an <a href='#...'> link is selected
     *
     * @param {string} hash_name The '#' reference to follow
     * @returns {number | undefined} The tab number selected, or null if the hash name was not known
     */
    select(name) {
        for (const t of this.tabs) {
            if (name == t.name) {
                return this.select_tab(t);
            }
        }
        if (this.tabs.length === 0) {
            return null;
        }
        return this.select_tab(this.tabs[0]);
    }
    select_tab(tab) {
        if (tab === this.selected_tab) {
            return tab.name;
        }
        for (const t of this.tabs) {
            t.set_hidden(t !== tab);
        }
        this.selected_tab = tab;
        this.callback(tab.client, tab.name);
        return tab.name;
    }
}
