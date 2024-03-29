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

(* Documentation

  The calibration diagram is 4 points of X axis and 5 of Y axis with also point (1,1).

  Assume the diagram is in the Z=0 plane.

  A camera at P with quaternion Q will have camera-relative coordinates Q.(xy0+P) = xyz'

  This has a pitch/roll and hence view XY

  As a guess one has XY = fov_scale * xyz' / z' (This assumes a type of lens)

  We should have points (0,0), (0,1), (0,2), (0,3) ...

  These have coords
  xyz00' = Q.000+Q.P
  xyz01' = Q.010+Q.P = xyz00' + 1*Q.dx010
  xyz02' = Q.020+Q.P = xyz00' + 2*Q.dx010
  xyz03' = Q.030+Q.P = xyz00' + 3*Q.dx010

  Now if Q.dx010 = dx,dy,dz then we have
  XY00 = xyz00' * (scale/z00') hence xyz00' = XY00/(scale/z00')
  XY01 = ((XY00 / (scale/z00')) + (dx,dy)) * scale / (z00'+dz)
       = ((XY00 * z00' +   (dx,dy)*scale) / (z00'+dz)
  XY02 = ((XY00 * z00' + 2*(dx,dy)*scale) / (z00'+2*dz)
  XY03 = ((XY00 * z00' + 3*(dx,dy)*scale) / (z00'+3*dz)

  let z = z00' and (dx,dy)*scale=DXY and XY00=XY

  Hence:
  XY01-XY00 = ((XY * (z-z-dz) + dxysc) / (z+dz)
            = (DXY - dz * XY) / (z+dz)
  and
  XY03-XY02 = ((XY*z + 3DXY) / (z+3dz) - ((XY*z + 2DXY) / (z+2dz)
            = XY*z*(1/(z+3dz) - 1/(z+2dz)) + DXY*(3/(z+3dz)-2/(z+2dz))

  1/(z+3dz)-1/(z+2dz) = (z+2dz-z-3dz)/(z+3dz)/z+2dz) = -dz/(z+3dz)/z+2dz)
  3/(z+3dz)-2/(z+2dz) = (3z+6dz-2z-2dz)/(z+3dz)/z+2dz) = z/(z+3dz)/z+2dz)

  XY03-XY02 = ((XY*z + 3DXY) / (z+3dz) - ((XY*z + 2DXY) / (z+2dz)
            = (DXY-dz*XY) * z/(z+3dz)/(z+2dz)
  Now z/(z+3dz)/z+2dz) = z / (z**2 + 5z.dz + 6.dz**2)
  If dz<<z then this = 1 / (z + 5.dz)
  XY03-XY02 = (DXY-dz*XY) / (z+5dz)

  xyz00' = (z+0*dz) * (XY00,1) = P + 0*Q.dx010
  xyz01' = (z+1*dz) * (XY01,1) = P + 1*Q.dx010
  xyz02' = (z+2*dz) * (XY02,1) = P + 2*Q.dx010
  xyz03' = (z+3*dz) * (XY03,1) = P + 3*Q.dx010

  Q.dx010 = (z+3*dz) * (XY03,1) - (z+2*dz) * (XY02,1)

  To a first approximation this is

  Q.dx010 = (z+5/2*dz) * ((XY03,1)-(XY02,1))

C0, about 54cm from the origin on the screen (C1 is 46cm)

Y axis  (374.591667 300.550000 ) (374.120000 224.720000 ) (375.580000 156.230000 ) (375.598592 86.098592 ) (375.085366 21.048780 )
X axis  (231.333333 129.294118 ) (375.580000 156.230000 ) (504.053398 175.679612 ) (619.271084 195.301205 )

(54.591667   60.550000 ) (0,+76)
(54.120000  -15.280000 ) (0,+70)
(55.580000  -83.770000 ) (0,+70)
(55.598592 -153.910000 ) (0,+65)
(55.085366 -218.950000 )

(-89.67     -110.71 )
( 55.580000 -83.77 )
(184.053398 -64.32 )
(299.271084 -44.69 )

Another way to look at it is that each point on the calibration is on a line from the camera out.
i.e. xyz00' = k0 * Dir(XY00)
And we know that
xyz01' - xyz00' =   dxyz01 = k1 * Dir(XY01) - k0 * Dir(XY00)  (3 equations, 5 unknowns)
and
xyz02' - xyz00' = 2*dxyz01 = k2 * Dir(XY02) - k0 * Dir(XY00)  (6 equations, 6 unknowns)

If we assume that k0=1 then 
xyz01' - xyz00' =   dxyz01 = k1 * Dir(XY01) - Dir(XY00)
xyz02' - xyz00' = 2*dxyz01 = k2 * Dir(XY02) - Dir(XY00)
xyz02' - xyz00' =   dxyz01 = k2/2 * Dir(XY02) - 1/2*Dir(XY00) = k1 * Dir(XY01) - Dir(XY00)
k2/2 * Dir(XY02) - k1 * Dir(XY01) = 1/2 Dir(XY00)
 *)
(*a Libraries *)
open Atcflib

(*a Point, line segment, and line, for calibration *)
(*m Point - x,y on a list of line segments *)
module rec Point : sig
  type t
  val make : float -> float -> t
  val get_xy : t -> Vector.t
  val add_segment : t -> Line_segment.t -> int -> unit
  val line_segment_connecting_to : t -> t -> Line_segment.t option
  val num_segments : t -> int
  val lines_through : t -> Line.t list
  val filter_parallel_to : float -> t -> t -> t list -> t list
  val str : t -> string
  val str_list : t list -> string
end = struct
  (*f type *)
  type t = {
      pos: Vector.t;
      mutable segments : (Line_segment.t * int) list;
    }

  (*f make  x y *)
  let make x y =
    let pos = Vector.make2 x y in
    { pos; segments = [] }

  (*f get_xy t -> Vector.t *)
  let get_xy t =
    t.pos

  (*f add_segment t ls n -> unit adds a line segment to this point *)
  let add_segment t ls n =
    t.segments <- (ls,n)::t.segments

  (*f num_segments t -> int gives number of line segments this point is part of *)
  let num_segments t = 
    List.length t.segments

  (*f line_segment_connecting_to t oth -> ls option gives the line segment that this point and the other point form (if any) *)
  let line_segment_connecting_to t oth =
    let connects_to acc (ls,n) =
      if (Line_segment.get_point ls (1-n)) == oth then
        Some ls
      else
        acc
    in
    List.fold_left connects_to None t.segments
    
  (*f lines_through t -> line list gives the list of lines that pass through this point (i.e. the lines that use any of the segments this point is part of *)
  let lines_through t =
    List.fold_left (fun acc (ls,_) -> Line_segment.accum_lines acc ls) [] t.segments
    
  (*f nearness_parallel_to xy:vec2 dxy:vec2 pt:vec2 -> float (1 = parallel, 0 = perpendicular)

  Determine if pt-xy is || to dxy (a unit vector)
   *)
  let nearness_parallel_to xy dxy pt =
    let pt_m_xy = Vector.(normalize (sub xy (copy pt))) in
    let correlation = Vector.dot_product pt_m_xy dxy in (* 1 or -1 is parallel *)
    (* Printf.printf "xy %s pt %s dxy %s corr %f\n" (Vector.str xy) (Vector.str pt) (Vector.str dxy) (abs_float correlation); *)
    abs_float correlation

  (*f filter_parallel_to closeness p0 p1 pts - filter a list of points to get those whose direction relative to p0 is close to p1's direction from p0 *)
  let filter_parallel_to closeness p0 p1 pts =
    let dxy = Vector.(normalize (sub p1.pos (copy p0.pos))) in
    let close_enough pt = (nearness_parallel_to p0.pos dxy pt.pos) > closeness in
    List.filter close_enough pts

  (*f str t - get string representation of the point *)
  let str t =
    Printf.sprintf "(%s)" (Vector.str t.pos)

  (*f str_list tl - get string representation of a list of points *)
  let str_list tl =
    List.fold_left (fun acc t->Printf.sprintf "%s %s" acc (str t)) "" tl

end
(*m Line_segment - pair of points as part of a list of lines *)
and Line_segment : sig
  type t
  val make : Point.t -> Point.t -> t
  val make_or_find : Point.t -> Point.t -> t
  val add_to_line : t -> Line.t -> unit
  val get_point : t -> int -> Point.t
  val get_other_point : t -> Point.t -> Point.t
  val accum_lines : Line.t list -> t -> Line.t list
  val str : t -> string
end = struct
  (*f type *)
  type t = {
    p0 : Point.t;
    p1 : Point.t;
    mutable lines: Line.t list;
    }

  (*f make p0 p1 -> t - make a line segment belonging to no lines; add the segment to the points *)
  let make p0 p1 =
    let t = {
      p0; p1; lines=[]
      } in
    Point.add_segment p0 t 0;
    Point.add_segment p1 t 1;
    t

  (*f make_or_find p0 p1 -> t - find the line segment from p0 to p1 (from those segments for p0) or make it if not there already *)
  let make_or_find p0 p1 =
    match (Point.line_segment_connecting_to p0 p1) with
      Some t -> t
    | None -> make p0 p1

  (*f get_point t n -> Point.t - get p0 or p1 from a line segment *)
  let get_point t n =
    if (n=0) then t.p0 else t.p1

  (*f get_other_point t pt -> Point.t - get the other point (assuming pt is either p0 or p1) on a line segment *)
  let get_other_point t pt =
    if (t.p0 == pt) then t.p1 else t.p0

  (*f add_to_line t l -> unit - add the line segment to a line *)
  let add_to_line t l =
    t.lines <- l::t.lines

  (*f accum_lines acc t -> acc - add all the lines that this segment is part of to acc, but only those that are not already part of acc *)
  let accum_lines acc t =
    let add_line_if_not_present acc l =
      if List.exists (fun ll -> l==ll) acc then acc else l::acc
    in
    List.fold_left add_line_if_not_present acc t.lines

  (*f str t -> string - get string representation of the line segment *)
  let str t =
    Printf.sprintf "%s->%s" (Point.str t.p0) (Point.str t.p1)
end
(*m Line - List of line segments with a midpoint vector and unit direction vector and a mean segment length*)
and Line : sig
  type t
  val make : Vector.t -> Vector.t -> float -> Line_segment.t list ->  t
  val get_dir_sc   : t -> Vector.t
  val num_pts : t -> int
  val filter_perpendicular_to : float -> t -> t list -> t list
  val pts_on_line : t -> bool -> Point.t list
  val str : t -> string
end = struct
  (*f type *)
  type t = {
      ls : Line_segment.t list;
      mid : Vector.t;
      dir : Vector.t;
      seg_mean : float;
    }

  (*f make mid dir mean ls - make a new line from a list of line segments and some stats *)
  let make mid dir seg_mean ls  =
    let t = {
        mid; dir; ls; seg_mean;
      } in
    List.iter (fun ls -> Line_segment.add_to_line ls t) ls;
    t

  (*f get_dir_sc t - get the direction of the line scaled by the segment mean (i.e. the expected vector from one point to the next) *)
  let get_dir_sc t = Vector.(scale t.seg_mean (copy t.dir))

  (*f how_perpendicular_to t oth - return a value of 0 if the two lines are perpendicular and 1 if parallel, with appropriate scaling between *)
  let how_perpendicular_to t oth =
    let dp = (Vector.dot_product t.dir oth.dir) in
    dp *. dp

  (*f filter_perpendicular_to closeness t tl -> filter the list of lines tl to those that are close enough to perpendicular to t *)
  let filter_perpendicular_to closeness t tl =
    List.filter (fun oth -> (how_perpendicular_to t oth) < closeness) tl
    
  (*f num_pts t -> int - get the number of points on the line *)
  let num_pts t =
    1 + (List.length t.ls)
    
  (*f pts_on_line t dirn - get a list of all the points on the line *)
  let pts_on_line t dirn =
    let p0 = Line_segment.get_point (List.hd t.ls) 0 in (* May be the wrong one! *)
    let add_other_point (last_pt,pts) l =
      let opt= Line_segment.get_other_point l last_pt in
      (opt,opt::pts)
    in
    let (_,pts) = List.fold_left add_other_point (p0,[p0]) t.ls in
    if dirn then pts else (List.rev pts)
    
  (*f str t -> string - get string representation of the line *)
  let str t =
    let r = Printf.sprintf "%s +- %s %f : " (Vector.str t.mid) (Vector.str t.dir) t.seg_mean in
    List.fold_left (fun acc l->Printf.sprintf "%s %s" acc (Line_segment.str l)) r t.ls
end
    
(*a Point_set *)
module Point_set = struct
  type t = {
      pts       : Point.t list;
      pts_lists : Point.t list list;
      mutable lines     : Line.t list;
    }

  let rec find_lines_of_n n closeness acc (pts:Point.t list) =
    let rec add_lines_of_n_from_pt acc (xy:Point.t) pts =
      match pts with
        xy1 :: tl ->
         if (List.length pts)>=(n-2) then
           let other_pts = Point.filter_parallel_to closeness xy xy1 tl in
           if (List.length other_pts)>=(n-2) then
             add_lines_of_n_from_pt ((xy::xy1::other_pts)::acc) xy tl
           else
             add_lines_of_n_from_pt acc xy tl
         else
           acc
      | _ -> acc
    in
    match pts with
      xy :: tl ->
       let new_acc = add_lines_of_n_from_pt acc xy tl in
       find_lines_of_n n closeness new_acc tl
    | _ ->
       acc

  let midpoint_of_line_set t =
    let vec_sum = Vector.make2 0. 0. in
    List.iter (fun pt -> ignore (Vector.add (Point.get_xy pt) vec_sum)) t;
    let n = List.length t in
    Vector.scale (1.0/.(float n)) vec_sum

  let direction_of_line_set t mid =
    let add_direction acc pt =
      let dxy = Vector.(normalize (sub mid (copy (Point.get_xy pt)))) in
      let must_invert = (Vector.dot_product acc dxy) < 0. in
      let dxy = if must_invert then (Vector.scale (-1.) dxy) else dxy in
      Vector.add dxy acc
    in
    let vec_sum = List.fold_left add_direction (Vector.make2 0. 0.) t in
    let vec_sum =
      if (Vector.get vec_sum 0)<0. then
        Vector.scale (-1.) vec_sum
      else
        vec_sum
    in
    let n = List.length t in
    Vector.scale (1.0/.(float n)) vec_sum

  let positions_on_line_set t mid dir =
    let k pt = Vector.(dot_product dir (sub mid (copy (Point.get_xy pt)))) in
    List.map k t

  let find_spacings_on_line_set t mid dir =
    let posns = positions_on_line_set t mid dir in
    match List.sort (fun (a:float) b -> compare a b) posns with
      x0::xs ->
       let (_,res) = List.fold_left (fun (lx,acc) x ->(x,(x-.lx)::acc)) (x0,[]) xs in
       res
    | _ -> []

  let find_spacing_on_line_set t mid dir =
    let spacings = find_spacings_on_line_set t mid dir in
    let n = List.length spacings in
    if n < 3 then (0., 0.) else (
      let sum     = List.fold_left (fun s x -> s+.x)      0. spacings in
      let sum_sq  = List.fold_left (fun s x -> s+.(x*.x)) 0. spacings in
      let mean    = sum /. (float n) in
      let var     = sum_sq /. (float n)  -. (mean *. mean) in
      let sd      = sqrt(var) in
      (mean, sd)
    )

  let create_line_segments t mid dir =
    let ta = Array.of_list t in
    let posns = positions_on_line_set t mid dir in
    (* nth element of t is at posn nth element of posns *)
    let ta_posns = Array.of_list (List.mapi (fun i x->(ta.(i),x)) posns) in
    Array.sort (fun (_,(ax:float)) (_,bx) -> compare ax bx) ta_posns;
    let (_,_,result) = Array.fold_left (fun (n,lpt,acc) (pt,_) -> (if (n = 0) then (1,pt,acc) else (n+1,pt,acc@[Line_segment.make_or_find lpt pt]))) (0,ta.(0),[]) ta_posns in
    result
    
  let build_lines t =
    let build_lines acc pts =
      let mid = midpoint_of_line_set pts in
      let dxy = direction_of_line_set pts mid in
      let (mean,sd) = find_spacing_on_line_set pts mid dxy in
      if (sd /. mean < 0.1) then (
        (* Printf.printf "%s + %s:  %s  : %f %f \n" (Vector.str mid) (Vector.str dxy) (Point_set.str pts) mean sd; *)
        let line_segments = create_line_segments pts mid dxy in
        let line = Line.make mid dxy mean line_segments in
        Printf.printf "%s\n" (Line.str line);
        line::acc
      ) else (
        acc
      )
    in
    let lines = List.fold_left build_lines [] t.pts_lists in
    t.lines <- lines;
    t

  let make_from_points n closeness pts =
    let pts_lists = find_lines_of_n n closeness [] pts in
    let t = {pts; pts_lists; lines=[]} in
    build_lines t

  let find_origins t : Point.t list =
    List.filter (fun p -> (Point.num_segments p)>=4) t.pts

  let find_axes_of_origin t p =
    Printf.printf "Trying origin %s\n" (Point.str p);
    match Point.lines_through p with
      [] -> None
    | (l0::ls) as l ->
       let perps = Line.filter_perpendicular_to 0.05 l0 ls in
       let find_longest_line (n,acc) l =
         let ns = (Line.num_pts l) in
         if ns > n then (ns, l) else (n,acc)
       in
       let (nperp,perp) = List.fold_left find_longest_line (0,l0) perps in
       let pars= Line.filter_perpendicular_to 0.05 perp l in
       let (npar,par) = List.fold_left find_longest_line (0,l0) pars in
       let dir0 = Line.get_dir_sc par in
       let dir1 = Line.get_dir_sc perp in
       let z = Vector.((get dir0 0)*.(get dir1 1) -. (get dir0 1)*.(get dir1 0)) in
       let must_reflect = z > 0. in
       Printf.printf "Z axis direction: %f (must reflect %b)\n" z must_reflect;
       Printf.printf "Length of par axis: %d and it is %s\n" npar (Line.str par);
       Printf.printf "Length of perp axis: %d and it is %s\n" nperp (Line.str perp);
       let dxys = [(-1.,-1.); (-1.,1.); (1.,-1.); (1.,1.)] in
       let get_corner (dx,dy) =
         (dx,dy,Vector.(add_scaled dx dir0 (add_scaled dy dir1 (copy (Point.get_xy p)))))
       in
       let corner_options = List.map get_corner dxys in
       let find_closest_to_corner c =
         let acc_closest_pt acc p =
           let (min_d,_) = acc in
           let d = Vector.(modulus (sub (Point.get_xy p) (copy c))) in
           if (d<min_d) then (d,(Point.get_xy p)) else acc
         in
         List.fold_left acc_closest_pt (1E20,c) t.pts
       in
       let accum_best_corner acc (dx,dy,c) =
         let (min_cd,_,_,_,_) = acc in
         let (min_d,p) = find_closest_to_corner c in
         if (min_d < min_cd) then (min_d,dx,dy,c,p) else acc
       in
       let (min_d,dx,dy,c,p) = List.fold_left accum_best_corner (1E20,0.,0.,dir0,dir0) corner_options in
       Printf.printf "Best option for point (1,1) in calibration space is %f,%f %s dist %f pt %s\n" dx dy (Vector.str c) min_d (Vector.str p);
       (* if dx,dy = 1,1 and must_reflect is false then:
        X axis is par in its current direction, so find origin and scan left 1 (if poss) for -1,0 and right 1 for (1,0) and right 2 (for 2,0)
        Y axis is perp in its current direction, so find origin and scan left 2 (if poss) for (0,-2) etc
        *)
       (* if dx,dy = 1,-1 and must_reflect is true then:
        X axis is par in its current direction, so find origin and scan left 1 (if poss) for -1,0 and right 1 for (1,0) and right 2 (for 2,0)
        Y axis is perp in reverse direction, so find origin and scan right 2 (if poss) for (0,-2) and left 2 for (0,2)
        *)
       (* if dx,dy = -1,-1 and must_reflect is false then:
        X axis is par in its current direction
        Y axis is perp in reverse direction
        *)
       (* if dx,dy = 1,1 and must_reflect is false then:
        X axis is perp in its current direction
        Y axis is par in current direction
        *)
       (* if dx,dy = -1,1 and must_reflect is true then:
        X axis is perp in its reverse direction
        Y axis is par in current direction


    dx  dy   mr    X       Y
     1   1   f   +par   +perp (x/y c2, -y/x c2, -y/x c1)
     -1  1   f   
     -1  -1  f   -par   -perp  (-x/-y c2, y/-x c2, y/-x c1)
     1  -1   f   
     1   1   t    
     -1  1   t   -par   -perp (-x/-y c1)
     -1  -1  t   
     1  -1   t   +par   +perp (x/y c1)

    So assume x is par, and origin is point p, and dx<0 means point (1,0) is point on par preceeding origin, etc
    We could do with a list of points on the Y axis, which is par, but in the correct order

        *)
       let x_axis = Line.pts_on_line par (dx<0.) in
       Printf.printf "X axis %s\n" (Point.str_list x_axis);
       let y_axis = Line.pts_on_line perp (dx<0.) in
       Printf.printf "Y axis %s\n" (Point.str_list y_axis);
       Some (x_axis, y_axis, c)

  let str t =
    List.fold_left (fun acc pt->Printf.sprintf "%s %s" acc (Point.str pt)) "" t.pts
    
end

(*a Mapping *)
module Mapping = struct
  type t =
    {
      mutable pt_sets     : Point_set.t list;
      mutable axes_sets   : int list;
      mutable pt_mappings : int list;
    }
  let make _ =
    { pt_sets     = [];
      axes_sets   = [];
      pt_mappings = [];
    }
  let add_pts t pts =
    let pt_sets = Point_set.make_from_points 4 0.98 pts in
    let origin_possibilities = Point_set.find_origins pt_sets in
    List.iter (fun p->ignore (Point_set.find_axes_of_origin pt_sets p)) origin_possibilities;
  ()
end
                   
(*f option_is_some/is_none/get *)
let option_is_none o = match o with None -> true | Some _ -> false
let option_is_some o = match o with None -> false | Some _ -> true
let option_get     x = match x with Some p -> p | None -> raise Not_found
let option_get_default o default = match o with None -> default | Some x -> x
let option_apply f d o = match o with None -> d | Some x -> f x

(* dxy_possibilites : line -> dist-in-cm -> camera-vec3 -> [] or [vec0; vec1]
  Find the k values along line whose distance from the camera-vec3 is dist-in-cm
  Return [] or [dxy0,dxy1] for the two vectors for the k values along the line
  Effectively 'assume line is perfectly directed and camera-vec3 is perfect, find
  the vectors along the line that match perfectly the dist-in-cm from camera-vec3
 *)
let dxy_possibilites l dist xyz0 =
  let ks = Camera.Line.ks_at_distance_from_point l dist xyz0 in
  match ks with
    None -> []
  | Some (k,kd) -> (
    let pa = Camera.Line.point_at_k l (k +. kd) in
    let pb = Camera.Line.point_at_k l (k -. kd) in
    [ Vector.(normalize (sub xyz0 pa));
      Vector.(normalize (sub xyz0 pb))]
  )

let dxys_parallel_to dxy dxy_option_lists =
  let rec add_most_parallel_to acc dxy_option_lists =
    match dxy_option_lists with
      [] -> Some acc
    | [] :: tl ->
       None
    | dxy_options :: tl ->
       let acc_most_parallel_to acc d =
         let (best, best_dxy) = acc in
         let p = Vector.dot_product d dxy in
         let p = p *. p in
         if p > best then (p, d) else acc
       in
       let _,best_dxy = List.fold_left acc_most_parallel_to (0., List.hd dxy_options) dxy_options in
       let new_acc = acc @ [best_dxy] in
       add_most_parallel_to new_acc tl
  in    
  add_most_parallel_to [] dxy_option_lists

let dxys_most_perp corr_dxys_x corr_dxys_y =
  let v = Vector.make3 0. 0. 0. in
  let most_perp_to_x acc (_,dxy_x) =
    let most_perp_to_y acc (_,dxy_y) =
      let (best,_) = acc in
      let p = Vector.dot_product dxy_x dxy_y in
      let p = p *. p in
      if p < best then (p, dxy_y) else acc
    in
    let best_p,best_dxy_y = List.fold_left most_perp_to_y (1., v) corr_dxys_y in
    let (p,_,_) = acc in
    if best_p<p then (best_p, dxy_x, best_dxy_y) else acc
  in
  List.fold_left most_perp_to_x (2., v, v) corr_dxys_x

(* average_dxys dxys -> dxy : calculate an 'average' vector
  Sums each vector after normalizing (and resolving to be in the same direction),
  yielding an approximately straight line of subvectors really.
  The average vector is a unit vector in that direction.

  Another option could be to find the vector (x,y,z) whose sum of squares of the dot product with
  the dxys is minimal
 *)
let average_dxys dxys =
  let dxy0 = List.hd dxys in
  let dxy = Vector.make3 0. 0. 0. in
  let add_dxy v =
    let dp = Vector.dot_product v dxy in
    let sc = (if dp<0. then (-1.) else (1.)) in
    ignore (Vector.add_scaled sc v dxy);
  in
  List.iter add_dxy dxys;
  let dp = Vector.dot_product dxy0 dxy in
  let sc = if (dp<0.) then (-1.) else 1. in
  Vector.(scale sc (normalize dxy))

(* correlate_dxys : dxy -> dxys -> float - get measure of how 'parallel' the list of dxys are to dxy
  Product ( dxys[i] _dot_product_ dxy ) for dxys[i] in dxys
 *)
let correlate_dxys dxy dxys =
  let correlate_dxy acc v =
    let dp = Vector.dot_product v dxy in
    acc *. dp
  in
  let c = List.fold_left correlate_dxy 1. dxys in
  abs_float c

(* best_at_k camera camera-line-through-calibration-origin distance-in-cm-to-origin (cm*scr_x*src_y)list
   
 *)
let best_at_k c l0 k0 data =
  (* xyz0 is the origin given k0 is the distance in cm for unit vector l0 *)
  let xyz0 = Camera.Line.point_at_k l0 k0 in
  (* add_dxys builds a list of [veca,vecb] pairs of XY vectors from origin xyz0
    where |(xyz0+veca|b)| = dist for veca/vecb corresponding to screen points (x,y) for distance dist
    (dist,x,y) should then be a distance in cm for a callibration point (X,Y) with dist |(X,Y)|
  *)
  let add_dxys acc (dist, x, y) =
    let l   = Camera.Camera.line_of_xy c x y in
    let dxy = dxy_possibilites l dist xyz0 in
    acc @ [dxy]
  in
  (* Get list of dxys for the given [ (dist,x,y) ] callibration points *)
  let dxy_poss = List.fold_left add_dxys [] data in
  (* dxy1 is *the list of candidate vectors for the first callibration point* *)
  let dxy1 = List.hd dxy_poss in
  (* best_for_acc dxy -> [float * dxy] list *)
  let best_for acc dxy =
    let best_dxy1a = option_get (dxys_parallel_to dxy dxy_poss) in
    let avg_dxy1a = average_dxys best_dxy1a in
    let corr_dxy1a = correlate_dxys avg_dxy1a best_dxy1a in
    Printf.printf "Best for %s : %f : %s : %s\n" (Vector.str dxy) corr_dxy1a (Vector.str avg_dxy1a) (Vector.str_list best_dxy1a);
    (corr_dxy1a, avg_dxy1a)::acc
  in
  List.fold_left best_for [] dxy1
  
let find_best_for_k k0 =

  let xfov = 43.0 in
  let c = Camera.Camera.make (Vector.make3 0. 0. 0.) 0. 0. 0. xfov in
  (* l0 is line through the origin *)
  let l0 = Camera.Camera.line_of_xy c (55.58)  (-83.77) in
  let data_xy = ((180.932099),(-127.37)) in
  (* data_y is the calibration figure points on the Y-axis - distance in cm from the origin and screen XY coordinates *)
  let data_y =  [ (5.,  (55.60),  (-153.91));
                (10., (55.60),  (-218.95));
                (5.,  (54.12),  (-15.28));
                (10., (54.59),  (60.55));
              ] in
  (* data_x is the calibration figure points on the X-axis - distance in cm from the origin and screen XY coordinates *)
  let data_x =  [ (10.,  (-89.67),  (-110.71));
                (10., (184.053398), (-64.32));
                (20., (299.271084), (-44.69));
                ] in
  (* coor_dxys_x is coor_dxy1a * avg_dxy1a 
  let corr_dxys_x = best_at_k c l0 k0 data_x in
  let corr_dxys_y = best_at_k c l0 k0 data_y in
  List.iter (fun (_,dxy_x) ->
      List.iter (fun (_,dxy_y) ->
          let dp = Vector.dot_product dxy_x dxy_y in
          let xyz110 = Camera.Line.point_at_k l0 k0 in
          Vector.add_scaled (-10.) dxy_x xyz110;
          Vector.add_scaled (5.) dxy_y xyz110;
          let (x,y)= Camera.Camera.xy_of_xyz c xyz110 in
          Printf.printf "(1,1) at %f,%f : %f : %s : %s \n" x y dp (Vector.str dxy_x) (Vector.str dxy_y)
        ) corr_dxys_y ) corr_dxys_x;
  let v,dxy_x,dxy_y = dxys_most_perp corr_dxys_x corr_dxys_y in
  Printf.printf "K0 %f: Perpendicularity %f for X axis %s Y axis %s\n" k0 v (Vector.str dxy_x) (Vector.str dxy_y)

let _ =
  find_best_for_k 54.0;
  find_best_for_k 55.0;
  find_best_for_k 56.0;
  find_best_for_k 57.0;
(*  find_best_for_k 58.0;
  find_best_for_k 59.0;
  find_best_for_k 60.0;
 *)
