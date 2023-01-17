(** Copyright (C) 2017,  Gavin J Stark.  All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * @file          track_glove.ml
 * @brief         Glove tracking
 *
 *)

open Batteries
open Atcflib
open Calibrate
open Maxlike
open Mapping
let data = Tracking_data.data2
open Find_camera_positions

let cs = [| cs0; cs1; cs2 |]

module Led =
  struct
    type t = { mutable xyz : Vector.t;
               mutable is_on: int;
               mutable is_tracking: int;
               camera_xys : Vector.t array;
             }
    let make num_cs = { xyz = Vector.make3 0. 0. 0.;
                        is_on = 0;
                        is_tracking = 0;
                        camera_xys = Array.init num_cs (fun _ -> Vector.make2 0. 0.);
                      }
    end
       
let camera_lines = Array.map (fun _ -> []) cs
let blah camera_pts = 
  let map_line (c,t,sx,sy) =
    let x = ((float sx)/.(float (2*t))) -. (float cs.(c).cx) in
    let y = ((float sy)/.(float     t)) -. (float cs.(c).cy) in
    let line = Camera.line_of_xy cs.(c) x y in
    camera_lines.(c) <- (line, x, y) :: camera_lines.(c) 
  in
  Array.iteri (fun i _ -> camera_lines.(i) <- []) cs;
  List.iter map_line camera_pts;
  let find_best_in_camera_1 i0 (l0, x0, y0) =
    let get_value_in_cameras i1 (l1, x1, y1) =
      let (midpoint, d, np) = Line.midpoint_between_lines l0 l1 in
      let d2 = Camera.distance_between_xys (cs.(0),cs.(1)) midpoint (x0,y0,x1,y1) in
      if (d2<400.0) then 
        Printf.printf "%2d %2d %14f %14f %8f  %s\n" i0 i1 d d2 np (Vector.str midpoint)
    in
    List.iteri get_value_in_cameras  camera_lines.(1)
  in
  List.iteri find_best_in_camera_1 camera_lines.(0);
  Printf.printf "\n";
  ()

let _ =
  List.iter blah data
