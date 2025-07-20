use std::collections::HashMap;
use std::rc::Rc;

use geo_nd::{quat, Quaternion, Vector};

use ic_base::json;
use ic_base::Quat;
use ic_base::{Point3D, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CameraDatabase, CameraInstance, CameraPolynomialCalibrate, CameraProjection};
use ic_image::{Color, Image, ImageRgb8};

use ic_mapping::{ModelLineSet, NamedPoint, NamedPointSet, PointMappingSet};

pub fn main() -> Result<(), String> {
    let camera_db_filename = "nac/camera_db.json";
    let camera_filename = "nac/camera_calibrate_6028.json";
    let read_filename: Option<&str> = None;
    let write_filename: Option<&str> = None;
    let read_filename = Some("/Users/gjstark/Git/Images/4V3A6028.JPG");
    let write_filename = Some("a.png");

    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut cdb: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    cdb.derive();
    let camera_json = json::read_file(camera_filename)?;
    let calibrate = CameraPolynomialCalibrate::from_json(&cdb, &camera_json)?;

    let mut cam = calibrate.camera().clone();
    cam.set_position([0., 0., 0.].into());
    cam.set_orientation(Quat::default());

    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    let pairings = calibrate.get_xy_pairings();

    let mut mls = ModelLineSet::new(&cam);

    for p0 in pt_indices {
        let (n0, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let dir0 = *grid_dir;
        let pm0 = pms.mapping_of_np(nps_of_pts.get(p0).unwrap()).unwrap();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n1, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dir1 = *grid_dir;
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let model0_xy = pairings[*n0].0;
            let model1_xy = pairings[*n1].0;
            let model0_xyz = [model0_xy[0], model0_xy[1], 0.].into();
            let model1_xyz = [model1_xy[0], model1_xy[1], 0.].into();
            let _ = mls.add_line_of_models(model0_xyz, model1_xyz, angle);
        }
    }

    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(1000, 1000);
    eprintln!("{best_cam_pos} {e}",);

    let best_cam_pos: Point3D = [13.76943098455281, -4.4539157030506376, 410.03914507909536].into();
    // let best_cam_pos: Point3D = [7.54435219975766, -2.2904012588912086, -407.86139540073606].into();

    // We can get N model direction vectors given the camera position,
    // and for each we have a camera direction vector
    let mut qs = vec![];
    for p0 in pt_indices {
        let (n, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let di_c = -*grid_dir;
        let model_xy = pairings[*n].0;
        let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
        let di_m = (best_cam_pos - model_xyz).normalize();
        let z_axis: Point3D = [0., 0., 1.].into();
        let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dj_c = -*grid_dir;
            let model_xy = pairings[*n].0;
            let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
            let dj_m = (best_cam_pos - model_xyz).normalize();

            let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
            let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
            let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m.as_ref()).into();

            let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
            let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
            let theta = theta_dj_m - theta_dj_c;
            let theta_div_2 = theta / 2.0;
            let cos_2theta = theta_div_2.cos();
            let sin_2theta = theta_div_2.sin();
            let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

            // At this point, qi_m * di_m = (0,0,1)
            //
            // At this point, q_z.conj * qi_m * di_m = (0,0,1)
            //                q_z.conj * qi_m * dj_m = dj_c_rotated
            //
            let q = qi_c.conjugate() * q_z * qi_m;

            // dc_i === quat::apply3(q.as_ref(), di_m.as_ref()).into();
            // dc_j === quat::apply3(q.as_ref(), dj_m.as_ref()).into();
            eprintln!(
                "di_c==q*di_m? {di_c} ==? {:?}",
                quat::apply3(q.as_ref(), di_m.as_ref())
            );
            eprintln!(
                "dj_c==q*dj_m? {dj_c} ==? {:?}",
                quat::apply3(q.as_ref(), dj_m.as_ref())
            );
            eprintln!("{q}");

            qs.push((1., q.into()));
        }
    }

    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();

    let mut camera = cam.clone();
    camera.set_position(best_cam_pos);
    camera.set_orientation(qr);

    // dbg!(&camera);
    let mut pts = vec![vec![], vec![], vec![], vec![]];
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (grid_xy, pxy_abs) in pairings {
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let model_txty = camera.world_xyz_to_camera_txty(model_xyz);
        let cam_txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
        // let pxy_rel = [pxy_abs[0] - 3590.0, 2235.0 - pxy_abs[1]].into();
        // let cam_txty2 = camera.px_rel_xy_to_txty(pxy_rel); // Uses projection
        // eprintln!("{cam_txty}, {cam_txty2} {model_txty}");
        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();
        world_yaws.push(model_ry.yaw());
        camera_yaws.push(cam_ry.yaw());
        if (model_ry.yaw() / cam_ry.yaw()) > 1.2 {
            continue;
        }
        let mut quad = 0;
        if cam_ry.cos_roll() < 0.0 {
            // X < 0
            quad += 1;
        }
        if cam_ry.sin_roll() < 0.0 {
            // Y < 0
            quad += 2;
        }
        pts[quad].push((cam_ry.yaw(), model_ry.yaw() / cam_ry.yaw() - 1.0));
    }

    let poly_degree = 5;
    let wts = polynomial::min_squares_dyn(poly_degree, &world_yaws, &camera_yaws);
    let stw = polynomial::min_squares_dyn(poly_degree, &camera_yaws, &world_yaws);
    let (max_sq_err, max_n, sq_err) =
        polynomial::square_error_in_y(&wts, &world_yaws, &camera_yaws);
    let avg_sq_err = sq_err / (world_yaws.len() as f64);

    if false {
        for i in 0..world_yaws.len() {
            let wy = world_yaws[i];
            let cy = camera_yaws[i];
            eprintln!(
                "{i} {wy} : {} : {cy} : {} : {wy}",
                wts.calc(wy),
                stw.calc(cy)
            );
        }
    }
    eprintln!(" wts: {wts:?}");
    eprintln!(" stw: {stw:?}");
    eprintln!(" avg sq_err: {avg_sq_err:.4e} max_sq_err {max_sq_err:.4e} max_n {max_n}");

    use poloto::build::PlotIterator;
    let plots = poloto::build::origin();
    let plot = poloto::build::plot("Quad x>0 y>0");
    let plot = plot.scatter(pts[0].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y>0");
    let plot = plot.scatter(pts[1].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x>0 y<0");
    let plot = plot.scatter(pts[2].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y<0");
    let plot = plot.scatter(pts[3].iter());
    let plots = plots.chain(plot);

    let mut wts_poly_pts = vec![];
    for i in 1..=100 {
        let world = (i as f64) * 0.35 / 100.0;
        let sensor = stw.calc(world);
        wts_poly_pts.push((world, sensor / world - 1.0));
    }
    let plot = poloto::build::plot("Wts Poly");
    let plot = plot.scatter(wts_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Yaw v Yaw", "x", "y"))
        .append_to(poloto::header().light_theme())
        .render_string()
        .map_err(|e| format!("{e:?}"))?;
    println!("{}", plot_initial);

    let xy_pairs = calibrate.get_xy_pairings();
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        let y_i = y_f as isize;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let x_i = x_f as isize;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba: Color = {
                if pt_indices.contains(&(x_i, y_i)) {
                    [100, 255, 100, 255]
                } else {
                    [0, 0, 0, 255]
                }
            }
            .into();
            pts.push((pt, rgba));
        }
    }
    if let Some(read_filename) = read_filename {
        let mut img = ImageRgb8::read_image(read_filename)?;
        if let Some(write_filename) = write_filename {
            let c = &[255, 0, 0, 0].into();
            for (_g, p) in &xy_pairs {
                img.draw_cross(*p, 5.0, c);
            }
            for (p, c) in &pts {
                let mapped = camera.map_model(*p);
                if c.0[0] == 100 {
                    let xyz = camera.world_xyz_to_camera_xyz(*p);
                    let txy = camera.world_xyz_to_camera_txty(*p);
                    eprintln!("{mapped} {xyz} {txy} {p} {c:?}");
                }
                img.draw_cross(mapped, 5.0, c);
            }
            img.write(write_filename)?;
        }
    }

    Ok(())
}
