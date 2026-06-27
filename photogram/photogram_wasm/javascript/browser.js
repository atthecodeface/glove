import { HtmlElement, Table } from "./html.js";
import * as file_set from "./file_set.js";
export class Browser {
    constructor(storage, browser, log) {
        this.browser = browser;
        this.log = log;
        // This will invoke the callback, so set everything else up first
        this.file_set = new file_set.FileSet(storage, this.repopulate.bind(this));
        this.file_set.get_file_list();
    }
    file_link(f) {
        // href is the file to fetch; donwload indicates it is to be downloaded to *that* filename
        const link = HtmlElement.new_ele("a", {}, [
            ["href", f],
            ["download", f],
        ]);
        link.ele.addEventListener("click", (event) => {
            const link = event.target;
            const data = this.file_set.load_file_as_str(f);
            link.href = "";
            if (data !== null) {
                link.href = "data:application/json," + encodeURIComponent(data) + "";
            }
        });
        link.add_content(f);
        return link;
    }
    repopulate() {
        this.browser.clear();
        this.browser.add_label("upload").add_content("Upload (Json)");
        this.browser.add_input_files(".json", true, this.upload_files.bind(this), {
            id: "upload",
        });
        this.add_table_of_cdb();
        this.add_table_of_projects();
        this.add_table_of_named_point_sets();
    }
    add_table_of_cdb() {
        const heading = HtmlElement.new_ele("h1", {
            classes: "browser_ft_heading",
        });
        heading.add_content("Camera Database");
        const table = new Table("cdb");
        table.add_headings(["Filename", "Bodies", "Lenses"]);
        for (const filename of this.file_set.files_of_kind(file_set.FileKind.Cdb)) {
            let obj = this.file_set.load_file_as_obj(filename, file_set.FileKind.Cdb);
            if (obj === null) {
                continue;
            }
            let cdb = obj;
            if (cdb === null) {
                continue;
            }
            var bodies_html = HtmlElement.new_ele("div");
            for (const b of cdb.bodies) {
                bodies_html.add_span(`${b.name}`);
                bodies_html.add_ele("br");
            }
            var lenses_html = HtmlElement.new_ele("div");
            for (const l of cdb.lenses) {
                lenses_html.add_span(`${l.name}`);
                lenses_html.add_ele("br");
            }
            const link = this.file_link(filename);
            table.add_body([link, bodies_html, lenses_html]);
        }
        this.browser.add_content(heading);
        this.browser.add_content(table.as_html());
    }
    add_table_of_projects() {
        const heading = HtmlElement.new_ele("h1", {
            classes: "browser_ft_heading",
        });
        heading.add_content("Projects");
        const table = new Table("proj");
        table.add_headings(["Filename", "Cdb", "Nps", "Number CIP"]);
        for (const filename of this.file_set.files_of_kind(file_set.FileKind.Project)) {
            let obj = this.file_set.load_file_as_obj(filename, file_set.FileKind.Project);
            if (obj === null) {
                continue;
            }
            let project = obj;
            if (project === null) {
                continue;
            }
            const link = this.file_link(filename);
            table.add_body([
                link,
                project.cdb,
                project.nps,
                project.cips.length.toString(),
            ]);
        }
        this.browser.add_content(heading);
        this.browser.add_content(table.as_html());
    }
    add_table_of_named_point_sets() {
        const heading = HtmlElement.new_ele("h1", {
            classes: "browser_ft_heading",
        });
        heading.add_content("Named point sets");
        const table = new Table("nps");
        table.add_headings(["Filename", "Number of points"]);
        for (const filename of this.file_set.files_of_kind(file_set.FileKind.Nps)) {
            let obj = this.file_set.load_file_as_obj(filename, file_set.FileKind.Nps);
            if (obj === null) {
                continue;
            }
            let nps = obj;
            if (nps === null) {
                continue;
            }
            const link = this.file_link(filename);
            table.add_body([link, nps.points.length.toString()]);
        }
        this.browser.add_content(heading);
        this.browser.add_content(table.as_html());
    }
    /*
  
      const nps_contents = [];
      for (const f of this.file_set.dir().files_of_type("nps")) {
        let t = this.file_set.load_file("nps", f);
        const obj = utils.parse_json(t);
        const num_pts = obj.length;
        nps_contents.push([f, `${num_pts}`]);
      }
        nps_contents,
      );
  
      const pms_contents = [];
      for (const f of this.file_set.dir().files_of_type("pms")) {
        let t = this.file_set.load_file("pms", f);
        const obj = utils.parse_json(t);
        const num_pts = obj.length;
        pms_contents.push([f, `${num_pts}`]);
      }
      this.create_file_table(
        "pms",
        "Point-mapping Sets",
        ["Filename", "Number of points"],
        pms_contents,
      );
  
      const cam_contents = [];
      for (const f of window.file_set.dir().files_of_type("cam")) {
        let t = this.file_set.load_file("cam", f);
        const obj = utils.parse_json(t);
        const cam_html =
          obj.body +
          "<br>" +
          obj.lens +
          "<br>Focus distance " +
          obj.mm_focus_distance +
          "mm";
        const posn_html =
          obj.position[0].toFixed(2) +
          ", " +
          obj.position[1].toFixed(2) +
          ", " +
          obj.position[2].toFixed(2);
        cam_contents.push([f, cam_html, posn_html]);
      }
      this.create_file_table(
        "cam",
        "Camera Placements",
        ["Filename", "Camera", "Position"],
        cam_contents,
      );
    }
  */
    /**
     * Upload files, invoked by a button from the HTML page itself
     *
     *  e.g. <input type="file" id="upload" name="upload" accept=".json" multiple>/>
     *
     * with a change event listener of  this.upload_files(<elemennt>.files);
     *
     * @param files
     */
    upload_files(files) {
        for (const file of files) {
            file.text().then((file_contents) => {
                this.upload_file_contents(file.name, file_contents);
            });
        }
    }
    upload_file_contents(filename, contents) {
        let file = file_set.UnknownFile.find_data_type(contents);
        if (file.kind() !== file_set.FileKind.Unknown) {
            this.log.info("upload", `Uploaded ${filename} of type ${file.kind()}`);
            this.file_set.save_file(filename, file, contents);
            // this.repopulate();
        }
        else {
            this.log.warning("upload", `Could not determine type of ${filename}`);
        }
    }
}
