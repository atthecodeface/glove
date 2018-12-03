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

(*a Libraries *)
open Batteries
open Atcflib

(*a Constants *)
let pi = 3.1415926538
let half_pi = pi /. 2.
let deg_to_rad = pi /. 180.
let rad_to_deg = 180. /.  pi

(*a modue Line *)
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
    let nonparallelity a b =
      let ab = Vector.dot_product a.direction b.direction in
      sqrt (1. -. ab *. ab )
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
      (midpoint, d, (nonparallelity a b))
    let str t =
      Printf.sprintf "[%s -> %s]" (Vector.str t.start) (Vector.str t.direction)
  end

(*a module Camera *)
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

    points are vec2s in screen space -w/2->w/2, -h/2->h/2
     *)
    (*t type t *)
    type t= {
        cx:int;
        cy:int;
        width:int;
        xyz:Vector.t;
        mutable q:Quaternion.t;
        inv_q:Quaternion.t;
        mutable fov:float;
        mutable pts:Vector.t list;
      }
    (*f update_q *)
    let update_q t =
      ignore (Quaternion.(conjugate (assign t.q t.inv_q)));
      (*Printf.printf "q %s inv_q %s\n" (Quaternion.str t.q) (Quaternion.str t.inv_q);*)
      ()
    (*f set_q *)
    let set_q t i j k =
      let r = sqrt (3. -. i*.i -. j*.j -. k*.k) in
      t.q <- Quaternion.make_rijk r i j k;
      ignore Quaternion.(scale (1. /. (modulus t.q)) t.q);
      update_q t
    (*f make *)
    let make xyz i j k xfov =
      let t = {cx=320;
       cy=240;
       width=640;
       xyz;
       q=Quaternion.make_rijk 1. 0. 0. 0.;
       inv_q=Quaternion.make_rijk 1. 0. 0. 0.;
       fov=(deg_to_rad *. xfov);
       pts=[];
              } in
      set_q t i j k;
      t
    (*f get_point t n - get nth point in t *)
    let get_point t n =
      List.nth t.pts n
    (*f add_point t x y - add point x y (0->w, 0->h) to t *)
    let add_point t x y =
      let pt = Vector.make2 (x -. (float t.cx)) (y -. (float t.cy)) in
      t.pts <- t.pts @ [pt]
    (*f add_point_vec t xy - add point (x y) (0->w, 0->h) to t *)
    let add_point_vec t xy =
      let x = Vector.get xy 0 in
      let y = Vector.get xy 1 in
      add_point t x y
    (*f angles_of_xy t x y - get pitch/roll of xy in screen space - inverse lens projection
        The direction vector to the point is:
         ((0,0,-1) rotated by pitch around y) rotated by roll around z
     *)
    let angles_of_xy t x y =
      (* Perform inverse of lens projection for point (x,y) *)
      let pitch = (sqrt (x*.x +. y*.y) ) *. t.fov /. (float t.width) in
      let roll  = atan2 y x in
      (pitch, roll)
    (*f xy_of_angles t pr - get xy in screen space of pitch/roll - lens projection
     *)
    let xy_of_angles t (p,r) = 
      let d = p /. t.fov *. (float t.width) in
      let y = d *. (sin r) in
      let x = d *. (cos r) in
      (x,y)
    (*f pr_of_direction t v3 - get (p,r) of vector in camera space
     *)
    let pr_of_direction t v3 =
      let m = Vector.modulus v3 in
      let x = (Vector.get v3 0)/.m in
      let y = (Vector.get v3 1)/.m in
      let z = (Vector.get v3 2)/.m in
      let (x,y,z) = if (z>0.) then Float.(neg x, neg y, z) else Float.( x,  y, neg z) in
      let p = acos z in
      let r = atan2 y x in
      (p,r)
    (*f direction_of_pr t pr - get new unit vector in camera space of (p,r)
     *)
    let direction_of_pr t (p,r) =
      let x2 = (sin p) *. (cos r) in
      let y2 = (sin p) *. (sin r) in
      let z2 = Float.( neg (cos p) ) in
      Vector.make3 x2 y2 z2
    (*f direction_of_xy t x y - get new unit vector of (x,y) in screen space relative (relative to centre) to camera
      Rotate 0,0,-1 by pitch around y yields sin(pitch), 0, -cos(pitch)
      Rotate this by roll around z yields sin(pitch)*cos(roll), -sin(pitch).sin(roll), -cos(pitch)
     *)
    let direction_of_xy t x y =
      let (p,r) = angles_of_xy t x y in
      direction_of_pr t (p,r)
    (*f direction_of_pt t pt:vec2 - get new unit vector of pt in screen space relative (relative to centre) to camera
     *)
    let direction_of_pt t v2 =
      let x = Vector.get v2 0 in
      let y = Vector.get v2 1 in
      direction_of_xy t x y
    (*f line_of_direction t v3 - get new line in world space from camera in direction (v3 in camera space)
     *)
    let line_of_direction t v3 =
      let v3_rel = Vector.(apply_q t.q (copy v3)) in
      Line.make t.xyz v3_rel
    (*f line_of_xy t v3 - get new line in world space from camera through screen space (relative to centre)
     *)
    let line_of_xy t x y = 
      let v3_camera = direction_of_xy t x y in
      line_of_direction t v3_camera
    (*f line_of_pt t v2 - get new line in world space from camera through screen space (relative to centre)
     *)
    let line_of_pt t v2 = 
      let v3_camera = direction_of_pt t v2 in
      line_of_direction t v3_camera
    (*f direction_of_xyz t xyz - get new vector in camera space of (xyz in world space)
     *)
    let direction_of_xyz t xyz = 
      let v3_rel = Vector.(add_scaled (-1.0) t.xyz (copy xyz)) in (* subtract out camera location *)
      let v3_camera = Vector.(apply_q t.inv_q v3_rel) in
      v3_camera
    (*f pr_of_xyz t xyz - get (p,r) of (xyz in world space)
     *)
    let pr_of_xyz t xyz =
      let v3_camera = direction_of_xyz t xyz in
      pr_of_direction t v3_camera
    (*f xy_of_xyz t xyz - get xy in screen space of (xyz in world space)
     *)
    let xy_of_xyz t xyz =
      let pr = pr_of_xyz t xyz in
      xy_of_angles t pr
    (*f str *)
    let str t =
      Printf.sprintf "%s:%s" (Vector.str t.xyz) (List.fold_left (fun s p -> Printf.sprintf "%s (%s)" s (Vector.str p)) "" t.pts)
    (*f distance_between_xys *)
    let distance_between_xys cs xyz (x0, y0, x1, y1) =
      let (mx0,my0) = xy_of_xyz cs.(0) xyz in
      let (mx1,my1) = xy_of_xyz cs.(1) xyz in
      let d0 = sqrt ((x0 -. mx0) *. (x0 -. mx0) +. (y0 -. my0) *. (y0 -. my0)) in
      let d1 = sqrt ((x1 -. mx1) *. (x1 -. mx1) +. (y1 -. my1) *. (y1 -. my1)) in
      d0 +. d1
    (*f error_of_mapping_in_3d *)
    let error_of_mapping_in_3d cs (p0,p1) =
      let l0 = line_of_pt cs.(0) (get_point cs.(0) p0) in
      let l1 = line_of_pt cs.(1) (get_point cs.(1) p1) in
      let d = Line.distance_between_lines l0 l1 in
      abs_float d
    (*f error_of_mapping *)
    let error_of_mapping cs (p0,p1) =
      let p0 = (get_point cs.(0) p0) in
      let p1 = (get_point cs.(1) p1) in
      let l0 = line_of_pt cs.(0) p0 in
      let l1 = line_of_pt cs.(1) p1 in
      let (xyz, d, np) = Line.midpoint_between_lines l0 l1 in
      let (x0,y0) = xy_of_xyz cs.(0) xyz in
      let (x1,y1) = xy_of_xyz cs.(1) xyz in
      let d0 = Vector.(modulus (add_scaled (-1.) p0 (make2 x0 y0))) in
      let d1 = Vector.(modulus (add_scaled (-1.) p1 (make2 x1 y1))) in
      (d0 +. d1, np)
    (*f error_of_mapping_verbose *)
    let error_of_mapping_verbose cs (p0,p1) =
      let p0 = (get_point cs.(0) p0) in
      let p1 = (get_point cs.(1) p1) in
      let l0 = line_of_pt cs.(0) p0 in
      let l1 = line_of_pt cs.(1) p1 in
      let (xyz, d, np) = Line.midpoint_between_lines l0 l1 in
      let (x0,y0) = xy_of_xyz cs.(0) xyz in
      let (x1,y1) = xy_of_xyz cs.(1) xyz in
      let d0 = Vector.(modulus (add_scaled (-1.) p0 (make2 x0 y0))) in
      let d1 = Vector.(modulus (add_scaled (-1.) p1 (make2 x1 y1))) in
      Printf.printf "xyz=%s : np/d0/d1=%8f:%8f:%8f:(%f,%f):%s cf (%f,%f):%s\n" (Vector.str xyz) np d0 d1 x0 y0 (Vector.str p0) x1 y1 (Vector.str p1);
      (d0 +. d1, np)
    (*f error_of_mappings *)
    let error_of_mappings cs pt_maps =
      let acc_error_of_mapping so_far (p0,p1) =
        let (e,np) = error_of_mapping cs (p0,p1) in
        (* so_far +. e *. np *)
        so_far +. e
      in
      List.fold_left acc_error_of_mapping 0. pt_maps
    (*f find_min_error *)
    let find_min_error cs mappings =
      let find_min_error_step bi bj bk range =
        let result = ref (100000., (0., 0., 0.)) in
        let step = range /. 5. in
        for di=0 to 10 do
          for dj=0 to 10 do
            for dk=0 to 10 do
              let i = bi +. ((float (di-5)) *. step) in
              let j = bj +. ((float (dj-5)) *. step) in
              let k = bk +. ((float (dk-5)) *. step) in
              set_q cs.(1) i j k;
              let err = error_of_mappings cs mappings in
              (*Printf.printf "%8f %8f %8f : %f\n" i j k err;*)
              if (err<(fst (!result))) then result := (err, (i,j,k));
            done
          done
        done;
        !result
        in
        set_q cs.(0) 0. 0. 0.;
        let (e, (i,j,k)) = find_min_error_step 0. 0. 0. 1.0  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.3  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.05  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.02  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.005  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.002  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.0005  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        let (e, (i,j,k)) = find_min_error_step i j k 0.0002  in
        Printf.printf "Error %f i %f j %f k %f\n" e i j k;
        set_q cs.(1) i j k;
        List.iter (fun (p0,p1) -> let (e,np)=(error_of_mapping_verbose cs (p0,p1)) in Printf.printf "Mapping %d %d err %f\n" p0 p1 e) mappings;
        (e, (i,j,k))
  end

let test_me _ =
  let l0 = Line.make (Vector.make3 0. 0. 0.) (Vector.make3 0. 0. (-1.0)) in
  let l1 = Line.make (Vector.make3 1. 0. 0.) (Vector.make3 1. 1. ( 0.0)) in
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l0 l1);
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_between_lines l1 l0)

let test_me2 _ =
  let cs = [|Camera.make (Vector.make3  0. 0. 0.) 0. 0. 0. 42.0;
             Camera.make (Vector.make3 30. 0. 0.) 0. 0. 0. 42.0;
           |] in
    let test_p = Vector.make3 20. 10. (-60.) in
    let (p,r) = Camera.pr_of_xyz cs.(0) test_p in
    (* The following should be (for 20, 10, -60) about 20.38 degrees pitch and 26.6 degrees roll*)
    Printf.printf "Should be about 20.38 and 26.6 %f %f\n" (p *. rad_to_deg) (r *. rad_to_deg);
    let (x,y) = Camera.xy_of_xyz cs.(0) test_p in
    (* Since fov is 45 degrees (x,y) should be about 20.38/45*640 pixels from the centre (290 pixels)
     This is x of 260 and y of 130
     *)
    Printf.printf "Should be 260,130 %f %f\n" x y ;
    Camera.add_point cs.(0) (x +. 320.) (y +. 240.);
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
    Printf.printf "xy of line direction (%f,%f)\n" lx ly;
    Printf.printf "----------------------------------------\n";
    Printf.printf "Rotate around z by 180\n";
    (*Camera.set_q cs.(0) 0.707 0.707 0.; 90 degree rotation around xy *)
    Camera.set_q cs.(0) 0. 0. 1.; (* 180 degree rotation around z *)
    let (p,r) = Camera.pr_of_xyz cs.(0) test_p in
    Printf.printf "Should be about 20.38 and 26.6-180=-153.4 %f %f\n" (p *. rad_to_deg) (r *. rad_to_deg);
    let (x,y) = Camera.xy_of_xyz cs.(0) test_p in
    Printf.printf "Should be -260,-130: %f %f\n" x y ;
    let line_p = Camera.(line_of_pt cs.(0) (get_point cs.(0) 0)) in
    (* Line goes from (0,0,0) through test_p, so its direction should be k*test_p *)
    Printf.printf "Should be proportional except xy -ve %s %s\n" (Vector.str test_p) (Vector.str line_p.direction);
    Printf.printf "Hence should be equal except xy -ve";
    for i=0 to 2 do Printf.printf " %f" Vector.((get test_p i) /. (get line_p.direction i)); done;
    Printf.printf "\n";
    Printf.printf "----------------------------------------\n";
    Printf.printf "Rotate around z by 90\n";
    Camera.set_q cs.(0) 0. 0. 0.707; (* 90 degree rotation around z *)
    let (p,r) = Camera.pr_of_xyz cs.(0) test_p in
    Printf.printf "Should be about 20.38 and 26.6-90=-63.4 %f %f\n" (p *. rad_to_deg) (r *. rad_to_deg);
    let (x,y) = Camera.xy_of_xyz cs.(0) test_p in
    Printf.printf "Should be -260,-130: %f %f\n" x y ;
    let line_p = Camera.(line_of_pt cs.(0) (get_point cs.(0) 0)) in
    (* Line goes from (0,0,0) through test_p, so its direction should be k*test_p *)
    Printf.printf "Should be proportional except xy -ve %s %s\n" (Vector.str test_p) (Vector.str line_p.direction);
    Printf.printf "Hence should be equal except xy -ve";
    for i=0 to 2 do Printf.printf " %f" Vector.((get test_p i) /. (get line_p.direction i)); done;
    Printf.printf "\n";
    ()

let _ =
  test_me ();
  test_me2 ()

