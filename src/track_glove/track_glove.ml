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

let pi = 3.1415926538
let half_pi = pi /. 2.
let deg_to_rad = pi /. 180.
let rad_to_deg = 180. /.  pi

module Line =
  struct
    type t = { start:Vector.t ;
               direction:Vector.t}
    let make start direction =
      let direction = Vector.(normalize (copy direction)) in
      {start; direction}
    let distance_from_line t p = 
      (*
        rp = p relative to start
        rp.direction = amount of direction
        Hence rp = rp.direction * direction + perpendicular_vector
        perpendicular_vector = rp - rp.direction * direction
        And distance = mod(perpendicular_vector)
       *)
      let rp = Vector.(add_scaled (-1.0) p (copy t.start)) in
      let rp_along_d = Vector.(scale (dot_product rp t.direction) (copy t.direction)) in
      let rpr = Vector.(add_scaled (-1.0) rp_along_d rp) in
      Vector.modulus rpr
    let distance_between_lines a b = 
      let bvd = Vector.(assign_cross_product3 a.direction b.direction (copy a.direction)) in
      let sd  = Vector.(add_scaled (-1.0) b.start (copy a.start)) in
      abs_float (Vector.dot_product bvd sd)
    let midpoint_between_lines a b = 
      (*
        (define the line between midpoints to be c1<->c2, c1 on a, c2 on b)
        avb = vector between the closest points (perp to both line directions)
        (line between midpoints is intersection of these planes, call it p1<->p2)
        a_n = normal to plane containing a and avb = ad x (ad x bd), and (p-as).(ad x (ad x bd))=0 for all points on the plane
        b_n = normal to plane containing b and avb = bd x (bd x ad), and (p-bs).(bd x (bd x ad))=0 for all points on the plane
        (line between midpoints is intersection of these planes)
        (c1 - bs).(bd x (bd x ad))=0 and c1 = as + k * ad hence
        (as + k * ad - bs).(bd x (bd x ad))=0
        (k * ad).(bd x (bd x ad)) + (as-bs).(bd x (bd x ad))=0
        k * ad.(bd x (bd x ad)) = (bs-as).(bd x (bd x ad))
        k = (bs-as).(bd x (bd x ad)) / ad.(bd x (bd x ad)) 
        k = (bs-as).b_n / ad.b_n
        c1 = as + (bs-as).b_n / ad.b_n * ad
        and
        c2 = bs + (as-bs).a_n / bd.a_n * bd
       *)
      let avb = Vector.(assign_cross_product3 a.direction b.direction (copy a.direction)) in
      let a_n = Vector.(assign_cross_product3 a.direction avb (copy avb)) in
      let b_n = Vector.(assign_cross_product3 b.direction avb (copy avb)) in
      let bs_m_as = Vector.(add_scaled (-1.0) a.start (copy b.start)) in
      let c1 = Vector.(add_scaled ( 1. *. ((dot_product bs_m_as b_n) /. (dot_product a.direction b_n))) a.direction (copy a.start)) in
      let c2 = Vector.(add_scaled (-1. *. ((dot_product bs_m_as a_n) /. (dot_product b.direction a_n))) b.direction (copy b.start)) in
      let d = Vector.(modulus (add_scaled (-1.0) c2 (copy c1))) in
      let midpoint = Vector.(add_scaled 0.5 c2 (scale 0.5 c1)) in
      (midpoint, d)
    let str t =
      Printf.sprintf "[%s -> %s]" (Vector.str t.start) (Vector.str t.direction)
  end

module Camera =
  struct
    (*
    Each camera has an x,y,z position, a view direction and magnification (dx,dy,dz), and a unit up direction (perpendicular to dx,dy,dz) effectively given by a single value.
    Then every point in 3D space is mapped to a 2d XY location for the camera.
    We can characterize the camera as a 3d vector and a unit quaternion and a scale.
    Then camera(xy) = Perspective(Cs*Rotation_Matrix(Cq)*(LED XYZ - Cxyz))
    OR
    k*(camera(xy),1) = Cs*Rotation_Matrix(Cq)*(Lxyz - Cxyz)
    OR 
    k*(camera(xy),1) = Cs*Rotation_Matrix(Cq)*Lxyz + C'xyz
    OR
    k*(camera(xy),1) - C'xyz = Cs*Rotation_Matrix(Cq)*Lxyz
    [Cs*Rotation_Matrix(Cq)^-1] (k*(camera(xy),1) - C'xyz) = Lxyz
    M^-1 . (k.Cx k.Cy k 1) = Lx Ly Lz 1

    If we define a camera to be at the origin we and looking straight with scale 1 (to give a frame of reference) we have
    camera0(xy) = Perspective(LED XYZ), or LED XYZ = k.(cx cy 1) for some k
    Indeed, we can 
    If there are 2 cameras for the same LED(XYZ), the second camera now has 
    camera0(xy) = Cs
     *)
    type t= {cx:int; cy:int; width:int; xyz:Vector.t; mutable yrot:float; mutable zrot:float; mutable fov:float; mutable pts:Vector.t list }
    let make xyz zrot yrot xfov =
      {cx=320; cy=240; width=640; xyz; yrot; zrot; fov=(deg_to_rad *. xfov); pts=[]}
    let get_point t n =
      List.nth t.pts n
    let add_point t x y =
      let pt = Vector.make2 (x -. (float t.cx)) (y -. (float t.cy)) in
      t.pts <- t.pts @ [pt]
    let angles_of_pt t pt =
      (* Perform inverse of lens projection for point (x,y) *)
      let x = Vector.get pt 0 in
      let y = Vector.get pt 1 in
      let pitch = (sqrt (x*.x +. y*.y) ) *. t.fov /. (float t.width) in
      let roll  = atan2 y x in
      (pitch, roll)
    let xy_of_angles t pr = 
      (*
        Perform lens projection for pitch p roll r
        Pitch is actually yaw - atan( sqrt(x^2+y^2) / sqrt(x^2+y^2+z^2) )
        Basically we rotate (X,0) anticlockwise by roll
        where X is the pixel distance due to 'pitch'
        So if pitch is equal to FOV/2 then X is width/2
       *)
      let (p,r) = pr in
      let d = p /. t.fov *. (float t.width) in
      let y = d *. (sin r) +. (float t.cy) in
      let x = d *. (cos r) +. (float t.cx) in
      (x,y)
    let line_of_pt t pt = 
      (*
        Determine direction vector (x,y,z) for the point
        (for no real lens projection should be k*(x,y,1))
       *)
      let (p,r) = angles_of_pt t pt in
      let x = (sin p) *. (cos (r +. t.zrot)) in
      let y = (sin p) *. (sin (r +. t.zrot)) in
      let z = (cos p) in
      let x2 = x*.(cos t.yrot) +. z*.(sin t.yrot) in
      let z2 = z*.(cos t.yrot) -. x*.(sin t.yrot) in
      Line.make t.xyz (Vector.make3 x2 y z2 )
    let pr_of_xyz t xyz = 
      (* Determine pitch/roll of an xyz from the projection *)
      let d = Vector.modulus xyz in
      let x = Vector.get xyz 0 in
      let y = Vector.get xyz 1 in
      let z = Vector.get xyz 2 in
      let x2 = x*.(cos t.yrot) -. z*.(sin t.yrot) in
      let z2 = z*.(cos t.yrot) +. x*.(sin t.yrot) in
      let p = acos (z2 /. d) in
      let p = (if (p < half_pi) then p else (p -. pi)) in
      let r = (atan2 y x2) -. t.zrot in
      (p,r)
    let xy_of_xyz t xyz =
      (* Determine screen x,y of an xyz from the projection *)
      let pr = pr_of_xyz t xyz in
      xy_of_angles t pr
    let str t = Printf.sprintf "%s:" (Vector.str t.xyz)
  end

let _ =
  let l0 = Line.make (Vector.make3 0. 0. 0.) (Vector.make3 0. 0. (-1.0)) in
  let l1 = Line.make (Vector.make3 1. 0. 0.) (Vector.make3 1. 1. ( 0.0)) in
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l0 l1);
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l1 l0)

let cs = [|Camera.make (Vector.make3 0. 0. 0.) 0. 0. 45.0;
           Camera.make (Vector.make3 30. 0. 0.) 0.307 (-0.327) 45.0;
         |]
let _ =
  let pts = [(0,4,3152,928);(0,4,2832,1040);(0,4,3280,1168);(0,4,2576,1360);(0,4,3088,1408);(1,4,4112,736);(1,4,4208,816);(1,4,3312,928);(1,4,3728,1024);(1,4,3216,1280);(1,4,3600,1312);] in
  cs.(0).yrot <- 0.1;
  cs.(0).zrot <- 0.3;
  let test_p = Vector.make3 20. 10. (-60.) in
  let (p,r) = Camera.pr_of_xyz cs.(0) test_p in
  (* The following should be (for 20, 10, -60) about 20.38 degrees pitch and 26.6 degrees roll*)
  Printf.printf "Should be about 20.38 and 26.6 (if yrot/zrot are 0) %f %f\n" (p *. rad_to_deg) (r *. rad_to_deg);
  let (x,y) = Camera.xy_of_xyz cs.(0) test_p in
  (* Since fov is 45 degrees (x,y) should be about 20.38/45*640 pixels from the centre (290 pixels)
     This is x of 260 and y of 130, and if centred on 320,240 then this is 60/580, 110/370
   *)
  Printf.printf "Should be 60,110 (if yrot/zrot are 0) %f %f\n" x y ;
  Camera.add_point cs.(0) x y;
  let line_p = Camera.(line_of_pt cs.(0) (get_point cs.(0) 0)) in
  (* Line goes from (0,0,0) through test_p, so its direction should be k*test_p *)
  Printf.printf "Should be proportional %s %s\n" (Vector.str test_p) (Vector.str line_p.direction);
  Printf.printf "Hence should be equal";
  for i=0 to 2 do Printf.printf " %f" Vector.((get test_p i) /. (get line_p.direction i)); done;
  Printf.printf "\n";
  (* Line goes through test_p, so distance should be 0 *)
  Printf.printf "Should be zero %f\n" (Line.distance_from_line line_p test_p);
  (* Note *)
  Printf.printf "(x,y) = (%f,%f)\n" x y;
  let (lx, ly) = Camera.xy_of_xyz cs.(0) line_p.direction in
  Printf.printf "xy of line direction (%f,%f)\n" lx ly
