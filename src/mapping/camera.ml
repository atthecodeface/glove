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

(*a modue Line - line in 3D *)
module Line =
  struct
    type t = { start:Vector.t ;
               direction:Vector.t
             }
    (*f make start direction -> t - make a line *)
    let make start direction =
      let direction = Vector.(normalize (copy direction)) in
      {start; direction}

    (*f point_at_k t k -> v - get a new vector that is start + k * direction *)
    let point_at_k t k =
      Vector.(add_scaled k t.direction (copy t.start))

    (*f distance_from_point t v -> float - calculate distance between line and point *)
    let distance_from_point t v = 
      (* Line is every pt p + k.d;
         Vector relative to point is p-v + k.d
         Closest to origin at x where x.d = 0
         x = p-v + k.d
         x.d = (p-v).d + k = 0
         k = (v-p).d
         x = p-v + ((v-p).d) * d
         |x| = sqrt(x.x)
         x.x = (p-v + ((v-p).d) * d) . (p-v + ((v-p).d) * d)
         = (p-v).(p-v) - 2*((p-v).d)^2 + ((v-p).d)^2
         = (p-v).(p-v) - ((p-v).d)^2
       *)
      let pmv = Vector.(sub v (copy t.start)) in
      let k = -. (Vector.dot_product pmv t.direction) in
      let x = Vector.(add_scaled k t.direction pmv) in
      Vector.modulus x

    (*f ks_at_distance_from_point t dist v -> float k0 * float kd option - calculate k=k0+-kd where the distance between the point on the line and v equals d *)
    let ks_at_distance_from_point t dist v = 
      (* Line is every pt p + k.d;
         Vector relative to point on line is x = p-v + k.d
         Want x.x = dist*dist
         x = p-v + k.d
         x.x = k^2 + 2((p-v).d)*k + |p-v|^2
         k^2 + 2((p-v).d)*k + |p-v|^2 - dist*dist = 0
         k = ((v-p).d) +- sqrt( ((v-p).d)^2 + dist*dist - |p-v|^2 )
       *)
      let pmv = Vector.(sub v (copy t.start)) in
      let vmp_dot_d = -. (Vector.dot_product pmv t.direction) in
      let discriminant = (vmp_dot_d*.vmp_dot_d) +. (dist*.dist) -. (Vector.modulus_squared pmv) in
      if discriminant<0. then None else Some (vmp_dot_d, (sqrt discriminant))

    (*f distance_betweek_lines a b -> float - calculate distance between two lines *)
    let distance_between_lines a b = 
      let bvd = Vector.(assign_cross_product3 a.direction b.direction (copy a.direction)) in
      let sd  = Vector.(add_scaled (-1.0) b.start (copy a.start)) in
      abs_float (Vector.dot_product bvd sd)

    (*f nonparallelity a b -> float - get a measure of how nonparallel lines are sqrt(1-sq(direction dot product)) *)
    let nonparallelity a b =
      let ab = Vector.dot_product a.direction b.direction in
      sqrt (1. -. ab *. ab )

    (*f midpoint_between_lines a b -> pt - calculate midpoint between two (nonparallel) lines *)
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

    (*f str t -> string - get string representation of line *)
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
    (*f set_xyz *)
    let set_xyz t x y z =
      ignore (Vector.(set 0 x (set 1 y (set 2 z t.xyz))))
      
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
    (*f delete_points *)
    let delete_points t =
      t.pts <- []
      
    (*f get_point t n - get nth point in t *)
    let get_point t n =
      List.nth t.pts n
    (*f add_point t x y - add point x y (0->w, 0->h) to t
      Note that inverting Y does have an effect, but if
      the inversion is used for two cameras then the correlation
      will not be effected
     *)
    let invert_y = true
    let add_point t x y =
      let pt =
        if invert_y then
          Vector.make2 (x -. (float t.cx)) ((float t.cy) -. y)
        else
          Vector.make2 (x -. (float t.cx)) (y -. (float t.cy))
      in
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
      pitch must be from 0 to pi/2
      Hence cos(pitch) is +ve
      But pitch is +/-z, hence must do cos(mod(z)), by negating x,y,z if necessary
      Then since we are looking at the rotation of vector (0,0,-1), we negate x and y too
     *)
    let pr_of_direction t v3 =
      let m = Vector.modulus v3 in
      let x = (Vector.get v3 0)/.m in
      let y = (Vector.get v3 1)/.m in
      let z = (Vector.get v3 2)/.m in
      let (x,y,z) = if (z>0.) then Float.(neg x, neg y, z) else Float.( x,  y, neg z) in
      let pitch = acos z in
      let roll = atan2 y x in
      (pitch,roll)
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
    (*f line_of_xy t x y - get new line in world space from camera through screen space (relative to centre)
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
      let x = Vector.(apply_q  t.q (make3 1. 0. 0.)) in
      let y = Vector.(apply_q  t.q (make3 0. 1. 0.)) in
      let z = Vector.(apply_q  t.q (make3 0. 0. (-1.))) in
      Printf.sprintf "%s + (X:%s Y:%s -Z:%s) %s:%s" (Vector.str t.xyz) (Vector.str x) (Vector.str y) (Vector.str z) (Quaternion.str t.q) (List.fold_left (fun s p -> Printf.sprintf "%s (%s)" s (Vector.str p)) "" t.pts)
    (*f distance_between_xys *)
    let distance_between_xys (c0,c1) xyz (x0, y0, x1, y1) =
      let (mx0,my0) = xy_of_xyz c0 xyz in
      let (mx1,my1) = xy_of_xyz c1 xyz in
      let d0 = sqrt ((x0 -. mx0) *. (x0 -. mx0) +. (y0 -. my0) *. (y0 -. my0)) in
      let d1 = sqrt ((x1 -. mx1) *. (x1 -. mx1) +. (y1 -. my1) *. (y1 -. my1)) in
      d0 +. d1
    (*f error_of_mapping_in_3d *)
    let error_of_mapping_in_3d (c0,c1) (p0,p1) =
      let l0 = line_of_pt c0 (get_point c0 p0) in
      let l1 = line_of_pt c1 (get_point c1 p1) in
      let d = Line.distance_between_lines l0 l1 in
      abs_float d
    (*f error_of_mapping *)
    let use_error_sq = true
    let error_of_mapping ?verbose:(verbose=false) (c0,c1) (p0,p1) =
      let p0 = (get_point c0 p0) in
      let p1 = (get_point c1 p1) in
      let l0 = line_of_pt c0 p0 in
      let l1 = line_of_pt c1 p1 in
      if verbose then (Printf.printf "Line 0:%s line 1:%s\n" (Line.str l0) (Line.str l1));
      let (xyz, d, np) = Line.midpoint_between_lines l0 l1 in
      if (Vector.get xyz 2) > (-5.) then (100000., np) else (
      let (x0,y0) = xy_of_xyz c0 xyz in
      let (x1,y1) = xy_of_xyz c1 xyz in
      let d0 = Vector.(modulus (add_scaled (-1.) p0 (make2 x0 y0))) in
      let d1 = Vector.(modulus (add_scaled (-1.) p1 (make2 x1 y1))) in
      if verbose then (
        Printf.printf "xyz=%s : np/d0/d1=%8f:%8f:%8f:(%f,%f):%s cf (%f,%f):%s\n" (Vector.str xyz) np d0 d1 x0 y0 (Vector.str p0) x1 y1 (Vector.str p1)
      );
      if use_error_sq then
        (d0*.d0 +. d1*.d1, np)
      else
        (d0 +. d1, np)
      )

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
      let (c0,c1) = cs in
      let find_min_error_step bi bj bk range =
        let result = ref (100000., (0., 0., 0.)) in
        let step = range /. 5. in
        for di=0 to 10 do
          for dj=0 to 10 do
            for dk=0 to 10 do
              let i = bi +. ((float (di-5)) *. step) in
              let j = bj +. ((float (dj-5)) *. step) in
              let k = bk +. ((float (dk-5)) *. step) in
              set_q c1 i j k;
              let err = error_of_mappings cs mappings in
              (* Printf.printf "%8f %8f %8f : %f\n" i j k err; *)
              if (err<(fst (!result))) then result := (err, (i,j,k));
            done
          done
        done;
        !result
        in
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
        set_q c1 i j k;
        List.iter (fun (p0,p1) -> let (e,np)=(error_of_mapping ~verbose:true cs (p0,p1)) in Printf.printf "Mapping %d %d err %f\n" p0 p1 e) mappings;
        (e, (i,j,k))

    (*f find_min_error_translate *)
    let find_min_error_translate scale cs mappings =
      let (c0,c1) = cs in
      let x0 = Vector.get c1.xyz 0 in
      let y0 = Vector.get c1.xyz 1 in
      let z0 = Vector.get c1.xyz 2 in
      let find_min_error_step x0 y0 z0 range =
        let result = ref (100000., (0., 0., 0.)) in
        let step = scale *. range /. 5. in
        for di=0 to 10 do
          for dj=0 to 10 do
            for dk=0 to 10 do
              let x = x0 +. ((float (di-5)) *. step) in
              let y = y0 +. ((float (dj-5)) *. step) in
              let z = z0 +. ((float (dk-5)) *. step) in
              set_xyz c1 x y z;
              let err = error_of_mappings cs mappings in
              (* Printf.printf "%8f %8f %8f : %f\n" i j k err; *)
              if (err<(fst (!result))) then result := (err, (x,y,z));
            done
          done
        done;
        !result
        in
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 1.0  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.3  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.05  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.02  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.005  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.002  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.0005  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        let (e, (x0,y0,z0)) = find_min_error_step x0 y0 z0 0.0002  in
        Printf.printf "Error %f i %f j %f k %f\n" e x0 y0 z0;
        set_xyz c1 x0 y0 z0;
        List.iter (fun (p0,p1) -> let (e,np)=(error_of_mapping ~verbose:true cs (p0,p1)) in Printf.printf "Mapping %d %d err %f\n" p0 p1 e) mappings;
        (e, (x0,y0,z0))
  end

(*a Tests *)
let test_me _ =
  Printf.printf "Testing distance between lines (0 0 0) + k (0 0 1) and (1 0 0) + m (1 1 0)\n";
  let xyz000 = Vector.make3 0. 0. 0. in
  let xyz100 = Vector.make3 1. 0. 0. in
  let xyz110 = Vector.make3 1. 1. 0. in
  let xyz001 = Vector.make3 0. 0. 1. in
  let l0 = Line.make xyz000 xyz001 in
  let l1 = Line.make xyz100 xyz110 in
  Printf.printf "Should be 1/sqrt(2) %f\n" (Line.distance_between_lines l0 l1);
  Printf.printf "Should be 1/sqrt(2) %f\n" (Line.distance_between_lines l1 l0);
  Printf.printf "Should be 1.0 %f\n" (Line.distance_from_point l0 xyz100);
  Printf.printf "Should be 1/sqrt(2) %f\n" (Line.distance_from_point l1 xyz000);
  Printf.printf "Should be sqrt(2) %f\n" (Line.distance_from_point l0 xyz110);
  let show_distances msg l dist v =
    match Line.ks_at_distance_from_point l dist v with
      None -> Printf.printf "%s : None\n" msg
    | Some (k0,kd) ->
       Printf.printf "%s : %f +- %f\n" msg k0 kd
  in
  show_distances "Should be none" l0 0. xyz100;
  show_distances "Should be +-sqrt(3)" l0 2. xyz100;
  ()
  
let test_me2 _ =
  let cs = [|Camera.make (Vector.make3  0. 0. 0.) 0. 0. 0. 42.0;
             Camera.make (Vector.make3 30. 0. 0.) 0.1 0.3 0.5 42.0;
           |] in
  let xys = [(100,100); (100,200); (300,100);] in
  let check_angles_xy c (x,y) =
    let check (dx,dy) =
      let (p,r) = Camera.angles_of_xy c (float (x*dx)) (float (y*dy)) in
      let (nx,ny) = Camera.xy_of_angles c (p,r) in
      Printf.printf "x,y %d,%d p,r %f,%f nx,ny %f,%f\n" (x*dx) (y*dy) p r nx ny;
    in
    List.iter check [(1,1); (-1,1); (1,-1); (-1,-1);]
  in
  Printf.printf "Testing angles_of_xy\n";
  List.iter (fun xy -> check_angles_xy cs.(1) xy)  xys;
  let check_directions_xy c (x,y) =
    let check (dx,dy) =
      let (p,r) = Camera.angles_of_xy c (float (x*dx)) (float (y*dy)) in
      let xyz = Camera.direction_of_pr c (p,r) in
      let (np,nr) = Camera.pr_of_direction c xyz in
      let (nx,ny) = Camera.xy_of_angles c (np,nr) in
      let rx = Vector.get xyz 0 in
      let ry = Vector.get xyz 1 in
      let rz = Vector.get xyz 2 in
      Printf.printf "x,y %d,%d p,r %f,%f x,y,z %f,%f%f nx,ny %f,%f\n" (x*dx) (y*dy) p r rx ry rz nx ny;
    in
    List.iter check [(1,1); (-1,1); (1,-1); (-1,-1);]
  in
  Printf.printf "Testing directions_of_pr\n";
  List.iter (fun xy -> check_directions_xy cs.(1) xy)  xys;
    

    Printf.printf "Test pr_of_xyz and xy_of_xyz (uses camera direction)\n";
    let test_p = Vector.make3 20. 10. (-60.) in
    let (p,r) = Camera.pr_of_xyz cs.(0) test_p in
    (* The following should be (for 20, 10, -60) about 20.38 degrees pitch and 26.6 degrees roll*)
    Printf.printf "Should be about 20.38 and 26.6 %f %f\n" (p *. rad_to_deg) (r *. rad_to_deg);
    let (x,y) = Camera.xy_of_xyz cs.(0) test_p in
    (* Since fov is 45 degrees (x,y) should be about 20.38/45*640 pixels from the centre (290 pixels)
     This is x of 260 and y of 130
     *)
    Printf.printf "Should be 260,130 %f %f\n" x y ;

    Printf.printf "Test line_of_pt and distance_from_point\n";
    Camera.add_point cs.(0) (x +. 320.) (y +. 240.);
    let line_p = Camera.(line_of_pt cs.(0) (get_point cs.(0) 0)) in
    (* Line goes from (0,0,0) through test_p, so its direction should be k*test_p *)
    Printf.printf "Should be proportional %s %s\n" (Vector.str test_p) (Vector.str line_p.direction);
    Printf.printf "Hence should be equal";
    for i=0 to 2 do Printf.printf " %f" Vector.((get test_p i) /. (get line_p.direction i)); done;
    Printf.printf "\n";
    (* Line goes through test_p, so distance should be 0 *)
    Printf.printf "Should be zero %f\n" (Line.distance_from_point line_p test_p);
    (* Note *)
    Printf.printf "(x,y) = (%f,%f)\n" x y;
    let (lx, ly) = Camera.xy_of_xyz cs.(0) line_p.direction in
    Printf.printf "xy of line direction (%f,%f)\n" lx ly;
    Printf.printf "----------------------------------------\n";

    Printf.printf "Retest with camera rotated around z by 180\n";
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
    Printf.printf "Retest with camera rotated around z by 90\n";
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

let test _ =
  test_me ();
  test_me2 ()

