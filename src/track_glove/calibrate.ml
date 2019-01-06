(** Copyright (C) 2018,  Gavin J Stark.  All rights reserved.
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
 * @file          callibrate.ml
 * @brief         Callibrate the glove cameras
 *
 *)

(*a Libraries *)
open Atcflib
open Mapping
   
(*a Top level *)
let data_c0 = [(375.085366,21.048780);(375.598592,86.098592);(500.932099,112.629630);(231.333333,129.294118);(375.580000,156.230000);(504.053398,175.679612);(619.271084,195.301205);(374.120000,224.720000);(374.591667,300.550000);]

let data_c1 = [(424.000000,91.000000);(323.291139,91.626582);(319.303030,91.946970);(238.000000,94.000000);(239.000000,120.500000);(308.607143,192.035714);(309.320000,242.380000);(408.324561,243.561404);(502.091837,290.224490);(407.683908,291.620690);(206.736364,294.509091);(309.300000,293.300000);(308.120690,339.919540);(308.738095,387.873016);]

let data_c2 = [(262.436170,15.617021);(262.443820,85.528090);(421.916667,87.864583);(91.145669,157.937008);(425.553030,159.583333);(589.987603,160.371901);(260.602362,161.559055);(257.225888,237.456853);(256.870861,321.867550);]

(* c0 has 375 on its y-axis and (+140,+27) on its x-axis *)
(* c1 has 309 on its y-axis and 291-3 on its x-axis *)
(* c2 has 257 on its y-axis and 157-160 on its x-axis *)
let data = List.map (fun (x,y) -> Mapping.Calibrate.Point.make (-.x)  (-.y)) data_c1
let data = List.map (fun (x,y) -> Mapping.Calibrate.Point.make (-.y)  x) data_c1
let data = List.map (fun (x,y) -> Mapping.Calibrate.Point.make y  (-.x)) data_c1
let data = List.map (fun (x,y) -> Mapping.Calibrate.Point.make x y ) data_c0

(*

  The callibration is 4 dots in a row horizontally, 5 vertically, and a line of three dots between the top and left-most of these two rows (with the third dot being the average).

            x
         
            x     x
         
      x     x     x     x  
         
            x
         
            x  

So there should be five points in a row (equidistant) and perpendicular to that four points in a row (equidistant)
       
 *)

(* If we use too small a number for 'lines that are parallel' then we include points that make the line segments be non-conformant
   0.95 is +- 18 degrees 
   0.98 is +- 11 degrees 
   0.99 is +- 8 degrees 
 *)
let _ =
  let pt_sets = Mapping.Calibrate.Point_set.make_from_points 4 0.98 data in
  let origin_possibilities = Mapping.Calibrate.Point_set.find_origins pt_sets in
  List.iter (fun p->ignore (Mapping.Calibrate.Point_set.find_axes_of_origin pt_sets p)) origin_possibilities;
  ()

                                

                                 
