import { Draw } from "./draw.js";

interface DataRangeProperties {
  include_zero?: boolean;
  expand_factor?: number;
}

export interface DataPoint {
  x(): number;
  y(): number;
  color(): string;
  set_x(x:number):void;
  set_y(y:number):void;
}

export class DataXY implements DataPoint {
  xy: [number, number];
  constructor(x: number, y: number) {
    this.xy = [x, y];
  }
  x(): number { return this.xy[0]; }
  y(): number { return this.xy[1]; }
  color(): string {
    return "#FF0";
  }
  set_x(x: number): void { this.xy[0] = x; }
  set_y(y: number): void { this.xy[1] = y; }
}

export class DataXYC implements DataPoint {
  xy: [number, number];
  pt_color: string;
  constructor(x: number, y: number, color:string) {
    this.xy = [x, y];
    this.pt_color = color;
  }
  x(): number { return this.xy[0]; }
  y(): number { return this.xy[1]; }
  color(): string {
    return this.pt_color;
  }
  set_x(x: number): void { this.xy[0] = x; }
  set_y(y: number): void { this.xy[1] = y; }
}

export class DataRange<T extends DataPoint> {
  data: T[];

  constructor(data: T[] = []) {
    this.data = data;
  }

  push(pt: T) {
    this.data.push(pt);
  }

  private get_range(
    min: number,
    max: number,
    properties: DataRangeProperties = {},
  ): [number, number] {
    if (properties.expand_factor !== undefined) {
      const mid = (min + max) / 2;
      const diff = max - min;
      min = mid - (diff / 2) * properties.expand_factor;
      max = mid + (diff / 2) * properties.expand_factor;
    }
    if (properties.include_zero !== undefined && properties.include_zero) {
      if (min * max > 0) {
        min = Math.min(0, min);
        max = Math.max(0, max);
      }
    }
    return [min, max];
  }

  get_xrange(properties: DataRangeProperties = {}): [number, number] {
    let min = this.data[0]!.x();
    let max = this.data[0]!.x();
    for (const xy of this.data) {
      min = Math.min(min, xy.x());
      max = Math.max(max, xy.x());
    }
    return this.get_range(min, max, properties);
  }

  get_yrange(properties: DataRangeProperties = {}): [number, number] {
    let min = this.data[0]!.y();
    let max = this.data[0]!.y();
    for (const xy of this.data) {
      min = Math.min(min, xy.y());
      max = Math.max(max, xy.y());
    }
    return this.get_range(min, max, properties);
  }
}

interface TicProperties {
  spacing?: number;
  length?: number;
  show_grid?: boolean;
  label?: boolean;
  id?: string;
}

export class Tics {
  spacing: number = 1;
  length: number = 1;
  show_grid: boolean = false;
  label: boolean = false;

  constructor(properties: TicProperties) {
    if (properties.spacing !== undefined) {
      this.spacing = properties.spacing;
    }
    if (properties.length !== undefined) {
      this.length = properties.length;
    }
    if (properties.show_grid !== undefined) {
      this.show_grid = properties.show_grid;
    }
    if (properties.label !== undefined) {
      this.label = properties.label;
    }
  }
  /** Set the spacing to be a power of ten such that the range will have at least min_tics */
  set_spacing_of_range(range: [number, number], min_tics: number) {
    const delta = range[1] - range[0];
    const delta_spacing = delta / min_tics;
    let spacing = Math.pow(10, Math.floor(Math.log10(delta_spacing)));
    const min_rounded_up = Math.ceil(range[0] / spacing);
    const max_rounded_down = Math.floor(range[1] / spacing);
    if (max_rounded_down - min_rounded_up < min_tics - 1) {
      spacing = spacing / 10;
    }
    this.spacing = spacing;
  }
}

class LinearMap {
  ofs: number = 0;
  scale: number = 1;
  constructor(ofs: number, scale: number) {
    this.ofs = ofs;
    this.scale = scale;
  }
  set_mapping(x0: number, map_x0: number, x1: number, map_x1: number) {
    this.scale = (map_x1 - map_x0) / (x1 - x0);
    this.ofs = map_x0 - x0 * this.scale;
  }
  map(x: number): number {
    return this.ofs + x * this.scale;
  }
  inv(mapped_x: number): number {
    return (mapped_x - this.ofs) / this.scale;
  }
}

export class Plot {
  /** Origin in the drawing space at which to put the bottom left of the graph */
  graph_origin: [number, number] = [0, 0];

  /** Width and hwight in the drawing space for the graph */
  graph_wh: [number, number];
  xtics: Tics[] = [];
  ytics: Tics[] = [];

  /** Derived graph area inside the drawing space */
  graph_area: [number, number, number, number] = [0, 0, 0, 0];

  /** Derived graph relative to absolute */
  map_x_graph_rel_to_abs: LinearMap;
  map_y_graph_rel_to_abs: LinearMap;

  /** Map from Data space to graph relative space */
  map_x_data_to_graph_rel: LinearMap;
  map_y_data_to_graph_rel: LinearMap;

  /** The area of the data space that is shown in the graph_area on the drawing */
  data_area: [number, number, number, number] = [0, 0, 0, 0];
  constructor(wh: [number, number]) {
    this.graph_wh = wh;
    this.map_x_graph_rel_to_abs = new LinearMap(0, 1);
    this.map_y_graph_rel_to_abs = new LinearMap(0, 1);
    this.map_x_data_to_graph_rel = new LinearMap(0, 1);
    this.map_y_data_to_graph_rel = new LinearMap(0, 1);
  }

  /** Set the position of the bottom-left of the graph rectangle
   *
   */
  set_graph_origin(origin: [number, number]) {
    this.graph_origin = origin;
    this.derive_data();
  }

  set_graph_wh(grapH_wh: [number, number]) {
    this.graph_wh = grapH_wh;
    this.derive_data();
  }

  set_data_range(x0: number, y0: number, x1: number, y1: number) {
    this.map_x_data_to_graph_rel.set_mapping(x0, 0, x1, 1);
    this.map_y_data_to_graph_rel.set_mapping(y0, 0, y1, 1);
    this.derive_data();
  }

  derive_data() {
    this.graph_area = [
      this.graph_origin[0],
      this.graph_origin[1],
      this.graph_origin[0] + this.graph_wh[0],
      this.graph_origin[1] - this.graph_wh[1],
    ];
    this.map_x_graph_rel_to_abs.set_mapping(
      0,
      this.graph_area[0],
      1,
      this.graph_area[2],
    );
    this.map_y_graph_rel_to_abs.set_mapping(
      0,
      this.graph_area[1],
      1,
      this.graph_area[3],
    );
    this.data_area = [
      this.map_x_data_to_graph_rel.inv(0),
      this.map_y_data_to_graph_rel.inv(0),
      this.map_x_data_to_graph_rel.inv(1),
      this.map_y_data_to_graph_rel.inv(1),
    ];
  }

  generate_box(draw: Draw) {
    draw.extend([
      ["W", 4.0],
      ["S", "#fff"],
      ["b"],
      ["m", this.graph_area[0], this.graph_area[1]],
      ["l", this.graph_area[2], this.graph_area[1]],
      ["l", this.graph_area[2], this.graph_area[3]],
      ["l", this.graph_area[0], this.graph_area[3]],
      ["l", this.graph_area[0], this.graph_area[1]],
      ["s"],
    ]);
  }

  generate_line_plot<T extends DataPoint>(draw: Draw, data_range: DataRange<T>) {
    draw.extend([
      ["push"],
      ["W", 4.0],
      ["S", "#ff3"],
      [
        "C",
        this.graph_area[0],
        this.graph_area[3],
        this.graph_area[2],
        this.graph_area[1],
      ],
      ["b"],
    ]);
    for (const d of data_range.data) {
      const x = this.map_x_graph_rel_to_abs.map(
        this.map_x_data_to_graph_rel.map(d.x()),
      );
      const y = this.map_y_graph_rel_to_abs.map(
        this.map_y_data_to_graph_rel.map(d.y()),
      );
      draw.extend([["l", x, y]]);
    }
    draw.extend([["s"], ["pop"]]);
  }

  generate_pt_plot<T extends DataPoint>(draw: Draw, data_range: DataRange<T>) {
    draw.extend([
      ["push"],
      ["W", 2.0],
      [
        "C",
        this.graph_area[0],
        this.graph_area[3],
        this.graph_area[2],
        this.graph_area[1],
      ],
    ]);
    for (const d of data_range.data) {
      const x = this.map_x_graph_rel_to_abs.map(
        this.map_x_data_to_graph_rel.map(d.x()),
      );
      const y = this.map_y_graph_rel_to_abs.map(
        this.map_y_data_to_graph_rel.map(d.y()),
      );
      const color = d.color();
      draw.extend([["push"], ["t", x, y]]);
      draw.extend([["b"], ["S", color], ["m", -5, -5], ["l", 5,5], ["m", -5,5], ["L", 10,-10], ["s"]]);
      draw.extend([["pop"]]);
    }
    draw.extend([["pop"]]);
  }

  generate_tics(draw: Draw) {
    for (let i = 0; i < this.xtics.length; i++) {
      this.generate_x_tics(draw, i);
    }
    for (let i = 0; i < this.ytics.length; i++) {
      this.generate_y_tics(draw, i);
    }
  }

  generate_grid(draw: Draw) {
    for (let i = 0; i < this.xtics.length; i++) {
      this.generate_x_grid(draw, i);
    }
    for (let i = 0; i < this.ytics.length; i++) {
      this.generate_y_grid(draw, i);
    }
  }

  generate_labels(draw: Draw) {
    for (let i = 0; i < this.xtics.length; i++) {
      this.generate_x_labels(draw, i);
    }
    for (let i = 0; i < this.ytics.length; i++) {
      this.generate_y_labels(draw, i);
    }
  }

  iter_spacing(
    dmin: number,
    dmax: number,
    spacing: number,
    callback: (d: number, graph_rel: number) => void,
  ): void {
    let drange_min = Math.floor(dmin / spacing);
    let drange_max = Math.ceil(dmax / spacing);
    for (let i = drange_min; i <= drange_max; i++) {
      let d = i * spacing;
      let rel = (d - dmin) / (dmax - dmin);
      if (rel < 0 || rel > 1) {
        continue;
      }
      callback(d, rel);
    }
  }

  x_iter_spacing(
    spacing: number,
    callback: (d: number, graph_rel: number) => void,
  ): void {
    this.iter_spacing(this.data_area[0], this.data_area[2], spacing, callback);
  }

  y_iter_spacing(
    spacing: number,
    callback: (d: number, graph_rel: number) => void,
  ): void {
    this.iter_spacing(this.data_area[1], this.data_area[3], spacing, callback);
  }

  generate_x_tics(draw: Draw, level: number) {
    if (this.xtics.length <= level) {
      return;
    }
    const tics = this.xtics[level]!;
    draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
    this.x_iter_spacing(tics.spacing, (_d, graph_rel) => {
      let x = this.map_x_graph_rel_to_abs.map(graph_rel);
      draw.extend([
        ["m", x, this.graph_area[1]],
        ["L", 0, -tics.length],
      ]);
      draw.extend([
        ["m", x, this.graph_area[3]],
        ["L", 0, tics.length],
      ]);
    });
    draw.extend([["s"]]);
  }

  generate_x_labels(draw: Draw, level: number) {
    if (this.xtics.length <= level) {
      return;
    }
    const tics = this.xtics[level]!;
    if (!tics.label) {
      return;
    }
    // draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
    this.x_iter_spacing(tics.spacing, (d, graph_rel) => {
      let x = this.map_x_graph_rel_to_abs.map(graph_rel);
      draw.extend([["txt", x, this.graph_area[1] + 30, d.toPrecision(3)]]);
    });
  }

  generate_y_labels(draw: Draw, level: number) {
    if (this.ytics.length <= level) {
      return;
    }
    const tics = this.ytics[level]!;
    if (!tics.label) {
      return;
    }
    this.y_iter_spacing(tics.spacing, (d, graph_rel) => {
      let y = this.map_y_graph_rel_to_abs.map(graph_rel);
      draw.extend([["txt", this.graph_area[0] - 30, y, d.toPrecision(3)]]);
    });
  }

  generate_x_grid(draw: Draw, level: number) {
    if (this.xtics.length <= level) {
      return;
    }
    const tics = this.xtics[level]!;
    if (!tics.show_grid) {
      return;
    }
    draw.extend([["W", 1.0], ["S", "#ff3"], ["b"]]);
    this.x_iter_spacing(tics.spacing, (d, graph_rel) => {
      let x = this.map_x_graph_rel_to_abs.map(graph_rel);
      if (d == 0) {
        draw.extend([["s"], ["W", 2.0], ["b"]]);
      }
      draw.extend([
        ["m", x, this.graph_area[1]],
        ["l", x, this.graph_area[3]],
      ]);
      if (d == 0) {
        draw.extend([["s"], ["W", 1.0], ["b"]]);
      }
    });
    draw.extend([["s"]]);
  }

  generate_y_tics(draw: Draw, level: number) {
    if (this.ytics.length <= level) {
      return;
    }
    const tics = this.ytics[level]!;
    draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
    this.y_iter_spacing(tics.spacing, (_d, graph_rel) => {
      let y = this.map_y_graph_rel_to_abs.map(graph_rel);
      draw.extend([
        ["m", this.graph_area[0], y],
        ["L", tics.length, 0],
      ]);
      draw.extend([
        ["m", this.graph_area[2], y],
        ["L", -tics.length, 0],
      ]);
    });
    draw.extend([["s"]]);
  }

  generate_y_grid(draw: Draw, level: number) {
    if (this.ytics.length <= level) {
      return;
    }
    const tics = this.ytics[level]!;
    if (!tics.show_grid) {
      return;
    }
    draw.extend([["W", 1.0], ["S", "#ff3"], ["b"]]);
    this.y_iter_spacing(tics.spacing, (d, graph_rel) => {
      let y = this.map_y_graph_rel_to_abs.map(graph_rel);
      if (d == 0) {
        draw.extend([["s"], ["W", 2.0], ["b"]]);
      }
      draw.extend([
        ["m", this.graph_area[0], y],
        ["l", this.graph_area[2], y],
      ]);
      if (d == 0) {
        draw.extend([["s"], ["W", 1.0], ["b"]]);
      }
    });
    draw.extend([["s"]]);
  }
}
