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
open Mapping
let data = Tracking_data.data2
         
let _ =
  let l0 = Line.make (Vector.make3 0. 0. 0.) (Vector.make3 0. 0. (-1.0)) in
  let l1 = Line.make (Vector.make3 1. 0. 0.) (Vector.make3 1. 1. ( 0.0)) in
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l0 l1);
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l1 l0)

let cs = [|Camera.make (Vector.make3 0. 0. 0.) 0. 0. 0. 45.0;
           Camera.make (Vector.make3 30. 0. 0.) 0. 0. 0. 45.0;
         |]
module Camera_leds = struct
  type t = {
      c : int;
      n : int;
      mutable on : bool;
      mutable x : float;
      mutable y : float;
    }
  let make c num_leds =
    Array.init num_leds (fun n -> { c; n; x=(-100.); y=0.; on=false; } )
  let invalidate t =
    t.x <- (-100.)
  let is_valid t =
    t.x > (-10.)
  let distance t x y =
    if (is_valid t) then
      let lx = t.x in
      let ly = t.y in
      Some (sqrt ((x-.lx)*.(x-.lx)+.(y-.ly)*.(y-.ly)))
    else
      None
  let min_distance ts x y max =
    let next_min opt_min t =
      match distance t x y with
        None -> opt_min
      | Some d ->
         if (d>max) then opt_min else
           (match opt_min with
              None -> Some (t,d)
            | Some (_,min) -> if (d>min) then opt_min else Some (t,d)
           )
    in
    Array.fold_left next_min None ts
  let is_on t =
    t.on
  let set_on v t =
    t.on <- v
  let set_xy x y t =
    t.x <- x;
    t.y <- y
  let add_at_first ts x y =
    let l = Array.length ts in
    let rec add_at_next i =
      if (i>=l) then None else (
        let t=ts.(i) in
        if (is_valid t) then (
          add_at_next (i+1)
        ) else (
          set_xy x y t;
          set_on true t;
          Some t.n
        )
      )
    in
    add_at_next 0
  let iter ts f =
    Array.iter f ts
  let nth_as_vector ts f n =
    let is_nth_matching r t =
      match r with
        (_, Some _) -> r
      | (i, None) -> (
        if (f t) then (
          if (n=i) then (n, Some t) else (i+1, None)
        ) else (
          r
        )
      )
    in
    match Array.fold_left is_nth_matching (0, None) ts with
      (_, None) -> Vector.make2 0. 0.
    | (_, Some t) -> Vector.make2 t.x t.y
  let count_on ts =
    let count_if_on n t =
      if (t.on) then (n+1) else n
    in
    Array.fold_left count_if_on 0 ts
  let display_on ts =
    let display_if_on t =
      if (t.on) then (
        Printf.printf "(%d, %d, %f, %f);" t.c t.n t.x t.y
      )
    in
    Array.iter display_if_on ts

end
module Mapping = struct
  type t = {xy0 : Vector.t;
            xy1 : Vector.t;
            mutable confidence : float;
           }
  let temp = Vector.make2 0. 0.
  let corr_dist d =
    let c = 1.0 -. ((d *. d) /. 40.) in
    if (c<0.) then 0. else c
  let make xy0 xy1 = {
      xy0;
      xy1;
      confidence=1.0;
    }
  let correlation t0 t1 =
    let d0 = Vector.(modulus (add_scaled (-1.) t0.xy0 (assign t1.xy0 temp))) in
    let d1 = Vector.(modulus (add_scaled (-1.) t0.xy1 (assign t1.xy1 temp))) in
    (corr_dist d0) *. (corr_dist d1)
  let rec find_close_mapping ?min_c:(min_c=0.1) tl t  =
    match tl with
      [] -> None
    | hd :: tl -> (
      let c = correlation hd t in
      if (c>min_c) then (Some hd) else (find_close_mapping tl t)
    )
  let find_close_mapping_or_add ?min_c:(min_c=0.1) tl t  =
    match find_close_mapping ~min_c:min_c tl t with
      None -> t::tl
    | Some ct -> (ct.confidence <- ct.confidence +. 1.; tl)
  let str t =
    Printf.sprintf "[%s ; %s @ %f]" (Vector.str t.xy0) (Vector.str t.xy1) t.confidence
end
     
let find_best_from_mappings (c0,c1) mappings_01 =
  Printf.printf "find_best_from_mappings\n";
  List.iter (fun m->Printf.printf "%s\n" (Mapping.str m)) mappings_01;
  Camera.delete_points c0;
  Camera.delete_points c1;
  let add_mapping n (m:Mapping.t) =
    let num = Float.to_int (sqrt m.confidence) in
    Camera.(add_point_vec c0 m.xy0;add_point_vec c1 m.xy1);
    n+1
  in
  let n = List.fold_left add_mapping 0 mappings_01 in
  let mappings = List.init n (fun i->(i,i)) in

  Printf.printf "Camera 0 %s\n" (Camera.str c0);
  Printf.printf "Camera 1 %s\n" (Camera.str c1);

  let cs = (c0,c1) in
  let (e, (i,j,k)) = Camera.find_min_error cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str c1);

  let (e, (i,j,k)) = Camera.find_min_error_translate 20. cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str c1);

(*

  let (e, (i,j,k)) = Camera.find_min_error cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str (snd cs));

  let (e, (i,j,k)) = Camera.find_min_error_translate 0.2 cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str (snd cs));
 *)  

  let cs = (c1,c0) in
  Printf.printf "Swapping cameras\n";

  let (e, (i,j,k)) = Camera.find_min_error cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C0 = %s\n" (Camera.str c0);
  Printf.printf "C1 = %s\n" (Camera.str c1);
  
  let cs = (c0,c1) in
  Printf.printf "Swapping cameras back\n";
  Printf.printf "Apply c0.q to c1\n";
  let q0' = Quaternion.(conjugate (copy c0.q)) in
  ignore (Quaternion.premultiply q0' c1.q);
  ignore (Vector.apply_q q0' c1.xyz);
  ignore (Quaternion.premultiply q0' c0.q);
  Camera.update_q c0;
  Camera.update_q c1;
  Printf.printf "C0 = %s\n" (Camera.str c0);
  Printf.printf "C1 = %s\n" (Camera.str c1);

  let (e, (i,j,k)) = Camera.find_min_error cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str c1);

  let (e, (i,j,k)) = Camera.find_min_error_translate 20. cs mappings in
  let (e, (i,j,k)) = Camera.find_min_error cs mappings in
  Printf.printf "Best q %f %f %f (error %f)\n" i j k e;
  Printf.printf "C1 = %s\n" (Camera.str c1);

  c1

let find_rots _ =
  let num_leds = 6 in
  let num_cs = 3 in
  let max_move = 10. in
  (*
  let rec strip_until_leds_off d =
    match d with
      []::tl -> tl
    | _::tl  -> strip_until_leds_off tl
    | _ -> []
  in
  let t = strip_until_leds_off data in
  *)
  let cleds = Array.init num_cs (fun i -> Camera_leds.make i num_leds) in
  let cxy_of_tracking (c,t,sx,sy) = (c, ((float sx)/.(float t) /. 2.), ((float sy)/.(float t))) in
  let add_or_find tracking =
    let (c,x,y) = cxy_of_tracking tracking in
    match Camera_leds.min_distance cleds.(c) x y max_move with
      None -> let _ = (Camera_leds.add_at_first cleds.(c) x y) in
              ()
    | Some (t,d) ->
       (* if (Camera_leds.is_on t) *)
       Camera_leds.(set_on true t; set_xy x y t)
  in
  let mappings_01 = ref [] in
  let handle_frame_tracking cpts =
    match cpts with
      [] -> (Array.iter (fun cl -> Camera_leds.iter cl Camera_leds.invalidate) cleds
            )
    | _ -> (
      Array.iter (fun cl -> Camera_leds.iter cl (Camera_leds.set_on false)) cleds;
      List.iter add_or_find cpts;
      let ccnt = Array.map Camera_leds.count_on cleds in
      if (ccnt.(0)=1) && (ccnt.(1)=1) then (
        let m = Mapping.make (Camera_leds.(nth_as_vector cleds.(0) is_on 0)) Camera_leds.(nth_as_vector cleds.(1) is_on 0) in
        mappings_01 := Mapping.find_close_mapping_or_add ~min_c:0.01 !mappings_01 m
      )
    )
  in
  List.iter handle_frame_tracking data;
  let known_good_data = [ [|(479.333333,341.166667); (470.000000,256.000000); (253.000000,215.000000);|];
                          [|(532.300000,213.400000); (554.863636,88.909091); (206.000000,296.200000);|];
                          [|(428.000000,291.500000); (387.884615,203.384615); (291.500000,233.500000);|];
                          [|(497.833333,291.666667); (479.000000,194.000000); (247.500000,234.200000);|];
                          [|(430.833333,342.666667); (403.500000,263.500000); (285.000000,211.500000);|];
                        ] in
                        
  let mappings_01 = List.map (fun cxya -> Mapping.make (Vector.make2 (fst cxya.(0)) (snd cxya.(0))) (Vector.make2 (fst cxya.(1)) (snd cxya.(1)))) known_good_data in
  let mappings_02 = List.map (fun cxya -> Mapping.make (Vector.make2 (fst cxya.(0)) (snd cxya.(0))) (Vector.make2 (fst cxya.(2)) (snd cxya.(2)))) known_good_data in
  let fov = 41.0 in
  let cs = [|Camera.make (Vector.make3 0. 0. 0.) 0. 0. 0. fov;
             Camera.make (Vector.make3 30. 0. 0.) 0. 0. 0. fov;
             Camera.make (Vector.make3 45. 0. 0.) 0. 0. 0. fov;
           |] in
(*
  let c1 = find_best_from_mappings (cs.(0),cs.(1)) mappings_01 in
  let c2 = find_best_from_mappings (cs.(0),cs.(2)) mappings_02 in
 *)
  let c1 = cs.(1) in
  let c2 = cs.(2) in
  (cs.(0), c1,c2)

let cs0,cs1,cs2 = find_rots ()
                          
