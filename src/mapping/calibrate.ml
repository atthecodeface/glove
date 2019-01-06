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
                   
