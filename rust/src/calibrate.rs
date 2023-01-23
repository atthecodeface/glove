/*! Documentation

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

!*/
mod types;
use types::{Point2D, Point3D, Point4D, Quat};
mod projection;
pub use projection::Projection;
mod point_mapping;
pub use point_mapping::PointMapping;
mod camera;
pub use camera::{LCamera, Rotations};
mod model_data;
pub use model_data::*;

use geo_nd::Vector;
use geo_nd::{matrix, quat};

#[test]
fn test_optimize() {
    // let camera0 = LCamera::new(
    //     [-80., -120., 280.].into(), // 540 mm fromm model 280 for fov 35
    //     quat::look_at(&[-33., -30., -570.], &[0.10, -1., -0.1]).into(),
    // );
    let camera0 = LCamera::new(
        // for C0_DATA_ALL
        // [-196., -204., 435.].into(), // 540 mm fromm model origin?
        // for -201.77,-292.29,648.1
        [54.10, -32.0, 781.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C1_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();

    let da = 0.01_f64.to_radians();
    let rotations = Rotations::new(da);
    let mut cam = camera0;
    for _ in 0..100 {
        cam = cam.get_best_direction(&rotations, &mappings[0]).0;
    }
    let num = mappings.len();
    let mut worst_data = (1_000_000.0, 0, cam, 0.);
    for i in 0..100 {
        /*
        let mut last_n = cam.find_worst_error(&mappings).0;
        for i in 0..30 {
            let mut n = cam.find_worst_error(&mappings).0;
            dbg!(i, n, last_n);
            if n == last_n {
                last_n = (last_n + 1 + (i % (num - 1))) % num;
            }
            cam = cam
                .adjust_direction_while_keeping_one_okay(
                    &rotations,
                    &|c, m, n| m[n].get_sq_error(c),
                    &mappings,
                    last_n,
                    n,
                )
                .0;
            last_n = n;
        }
        for i in 0..100 {
            let best_n = cam.find_best_error(&mappings).0;
            let worst_n = cam.find_worst_error(&mappings).0;
            dbg!(i, best_n, worst_n);
            if best_n == worst_n {
                break;
            }
            cam = cam
                .adjust_direction_while_keeping_one_okay(
                    &rotations,
                    &|c, m, n| m[n].get_sq_error(c),
                    // &|c, m, n| c.total_error(&mappings),
                    // &|c, m, n| c.worst_error(&mappings),
                    &mappings,
                    best_n,
                    worst_n,
                )
                .0;
        }
         */
        for i in 0..num {
            cam = cam
                .adjust_direction_rotating_around_one_point(
                    // &|c, m, n| m[n].get_sq_error(c),
                    // &|c, m, n| c.total_error(&mappings),
                    &|c, m, n| c.worst_error(&mappings),
                    &mappings,
                    i,
                    0,
                )
                .0;
        }
        dbg!(
            "Total error pre move",
            cam.total_error(&mappings),
            cam.worst_error(&mappings)
        );
        dbg!(cam);
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        if true {
            cam = cam
                .adjust_position_in_out(&mappings, &|c, m| c.worst_error(m))
                .0;
            cam = cam.adjust_position(&mappings, &|c, m| c.worst_error(m)).0;
        }
        // Try for cam1 to get good rough position
        if false {
            cam = cam
                .adjust_position_in_out(&mappings, &|c, m| c.total_error(m))
                .0;
            cam = cam.adjust_position(&mappings, &|c, m| c.total_error(m)).0;
        }
        eprintln!("Loop {} completed", i);
        dbg!(
            "Total error post move",
            cam.total_error(&mappings),
            cam.worst_error(&mappings)
        );
        let we = cam.worst_error(&mappings);
        eprintln!("WE {:.2} Camera {}", we, cam);
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        if we < worst_data.0 {
            worst_data = (we, i, cam, cam.total_error(&mappings));
        }
    }
    eprintln!(
        "Lowest WE {} {:.2} {:.2} Camera {}",
        worst_data.1, worst_data.0, worst_data.3, worst_data.2
    );
    assert!(false);
}

fn test_calibrate() {
    let camera0 = LCamera::new(
        [-10., 20., 540.].into(), // 540 mm fromm model
        quat::look_at(&[-33., -130., -540.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C0_DATA
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let da = 0.02_f64.to_radians();
    let rotations = Rotations::new(da);
    let camera0 = camera0
        .get_best_direction(&rotations, mappings.last().unwrap())
        .0;
    for pm in mappings.iter() {
        pm.show_error(&camera0);
    }
    let mut cam = camera0.clone();
    cam = cam.adjust_position(&mappings, &|c, m| c.total_error(m)).0;
    for pm in mappings.iter() {
        pm.show_error(&cam);
    }
    assert!(false);
    // For the given direction and currrent estimate of position we can deduce the
    // view_xyz of each mapping model point [i]
    //
    // That gives an estimate of 'z' for view_xyz (z_est[i])
    //
    // Now, view_xyz[i] = direction * (model[i] - position),
    //
    // and scr_xy_est[i] = view_xyz.xy / z_est[i]
    //
    // Hence scr_xy_est[i] = direction/z_est[i] * model[i] - direction/z_est[i]*position
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (((d * m[i]) - s[i]*z_est[i]) - (d*p)).x ^ 2
    //
    //              = 1/(z_est[i]^2) * ((d[row 0] * m[i]) - s[i].x*z_est[i] - (d[row 0]*p)) ^ 2
    //
    // let blah = d[row 0] * m[i]) - s[i].x*z_est[i]
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (blah - d[row 0]*p) ^ 2
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (blah^2 - 2*blah*d[row 0]*p + (d[row 0]*p) ^ 2)
    //
    // d[row 0]*p   = d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z
    // d[row 0]*p^2 = d[0,0]*p.x ^ 2 + 2*d[0,0]*p.x*(d[1,0]*p.y + d[2,0]*p.z) + (d[1,0]*p.y + d[2,0]*p.z)^2
    //
    // d/d(p.x)[ d[row 0]*p ] = d[0,0]
    // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]*p.x + 2*d[0,0]*(d[1,0]*p.y + d[2,0]*p.z)
    // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]*(p.x + d[1,0]*p.y + d[2,0]*p.z)
    //
    // d/d(p.y)[ d[row 0]*p ] = d[1,0]
    // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*p.y + 2*d[1,0]*(d[0,0]*p.x + d[2,0]*p.z)
    // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*(p.y + d[0,0]*p.x + d[2,0]*p.z)
    //
    // dE_sq_x / d(p.x) = 1/(z_est[i]^2) * (2 * blahx[i] * d[0,0] + 2*d[0,0]*(p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.x) = 2*d[0,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.y) = 2*d[1,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.z) = 2*d[2,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    //
    // Similarly E_sq_y = 1/(z_est[i]^2) * (blahy^2 - 2*blahy*d[row 1]*p + (d[row 1]*p) ^ 2)
    // where blahy[i] =  d[row 1] * m[i]) - s[i].y*z_est[i]
    //
    // dE_sq_y / d(p.x) = 2*d[0,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    // dE_sq_y / d(p.y) = 2*d[1,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    // dE_sq_y / d(p.z) = 2*d[2,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    //
    // dE_sq = Sum(dE_sq_x[i]) + Sum(dE_sq_y[i])
    //
    // We can write dE_sq = M * p + v for some M and v
    //
    // Then when dE_sq = 0, M * p = -v and hence p = - M(inv) * v
    //
    // Sadly M is singular, as each row of the matrix M is a
    //
    // So for a given p estimate (which yields z_est and blahx/y/z[i] and hence M_est, in a sense)
    //
    // we can generate a new 'minimum square error' p
    let mut m = [0.0_f64; 9];
    let mut v = [0.0_f64; 3];
    for pm in mappings.iter() {
        pm.add_sq_error_mat(&camera0, &mut m, &mut v);
    }
    for mv in m.iter() {
        dbg!(1000. * mv);
    }
    // The matrix will be singular
    //
    // (matrix) * px = c0
    let mi = matrix::inverse3(&m);
    let new_p = matrix::transform_vec3(&mi, &v);
    dbg!(camera0.position());
    dbg!(new_p);
    assert!(false);
}
