import { WasmCip, WasmProject } from "../pkg/photogram_wasm.js";
export interface Application {
  current_project_name(): string | null;
  current_project(): WasmProject | null;
  current_cip(): WasmCip | null;
}
