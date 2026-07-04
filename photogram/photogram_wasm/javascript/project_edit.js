export class ProjectEdit {
    constructor(application, log, html_div) {
        this.application = application;
        this.log = log;
        this.html_div = html_div;
        this.ctl_div = this.html_div.add_ele("div");
    }
    repopulate() { }
}
