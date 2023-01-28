//a Modules
// use glove::calibrate::Projection;
use glove::calibrate::min_squares;
use glove::calibrate::{CalcPoly, CameraSensor, Point2D, RectSensor};

//a Tests
//fp test_min_sq
#[test]
fn test_min_sq() {
    let xi = [1., 2., 3., 4.];
    let yi = [1., 2.0, 3., 4.];
    let r = min_squares::<3, 9>(&xi, &yi);
    dbg!(r);
    // assert!(false);
}

//fp find_poly_from_bars
#[test]
fn find_poly_from_bars() {
    let width_20 = 6230.0;
    const BARS_20MM: &[f64] = &[
        112.0, 199.5, 284.0, 370.0, 456.0, 543.0, 629.5, 715.5, 804.0, 890.5, 979.0, 1064.5,
        1151.0, 1230.0, 1317.0, 1406.5, 1493.5, 1586.0, 1672.0, 1764.5, 1855.5, 1945.5, 2034.5,
        2125.5, 2218.5, 2305.5, 2401.5, 2493.0, 2585.0, 2675.5, 2768.0, 2857.5, 2955.0, 3044.5,
        3139.5, 3229.5, 3324.5, 3413.5, 3504.5, 3598.0, 3689.5, 3784.5, 3874.0, 3967.0, 4056.5,
        4148.0, 4238.5, 4330.0, 4418.0, 4508.0, 4598.0, 4686.5, 4776.5, -1., -1., -1., -1., 5213.0,
        5298.0, 5384.5, 5471.0, 5556.0, 5639.5, 5726.7, 5812.5, 5894.0, 5979.5, 6063.0, 6148.5,
    ];
    let camera_distance_mm = (42.0 + 10. / 16.0) * 25.4;
    let bar_width_mm = 31.5 * 25.4 / 40.0; //  # 20mm
                                           // let bar_width_mm = 20.0;
    let x_scale_20 = 5184.0 / width_20 * 20.0 / 22.3 * 2.0;
    let x_scale_20 = 1.0 / x_scale_20;

    let pixel_width = width_20;
    let equispaced_data = BARS_20MM;
    let poly_degree = 5;
    let x_scale = x_scale_20;

    let cx_pix = pixel_width / 2.0;

    // Find which entries of the equispaced_data are either side of the center pixel
    // This assumes they are sorted (but with -1.0 for those that are uncertain)
    let mut cr_index = 0;
    for (i, b) in equispaced_data.iter().enumerate() {
        if *b > cx_pix {
            cr_index = i;
            break;
        }
    }
    let cl_index = cr_index - 1;

    let cl_pix = equispaced_data[cl_index];
    let cr_pix = equispaced_data[cr_index];

    // Find the offset from cx_pix of the middle of the centre bar in pixels
    let ofs_pix_closest_halfway_bars_to_center = (cr_pix + cl_pix) / 2.0 - cx_pix;

    // Find the width of the centre bar
    let bar_width_pix_at_center = cr_pix - cl_pix;

    // Find the offset in mm assuming the bar width in mm
    //
    // The bar to left of center is at ofs_mm_center - bar_width_mm / 2
    // The bar to right of center is at ofs_mm_center + bar_width_mm / 2
    let ofs_mm_center =
        ofs_pix_closest_halfway_bars_to_center / bar_width_pix_at_center * bar_width_mm;
    dbg!(ofs_mm_center);

    let mut sample_data = vec![];
    let mut pixs = vec![];
    let mut xs = vec![];
    let mut ys = vec![];
    for (i, b_pix) in equispaced_data.iter().enumerate() {
        if *b_pix < 0. {
            continue;
        }
        // b_bar_num is 0 for the bar just to the left of center
        let b_bar_num = i as f64 - cl_index as f64; // Signed!
        let b_mm = (b_bar_num - 0.5) * bar_width_mm + ofs_mm_center;
        // Quite possible b_mm is perhaps a little tight on the left-hand side
        // and a bit wider on the right-hand side
        // Hence bar_width_mm is perhaps of the form a*bar_num+c
        // hence b_mm = (b_bar_num - 0.5) * (a * bar_num + c + bar_width_mm) + ofs_mm_center;
        let b_mm = (b_bar_num - 0.5) * (-0.00094 * b_bar_num - 0.21 + bar_width_mm) + ofs_mm_center;
        // let b_mm = (b_bar_num * 1.0001 - 0.5) * (0. + bar_width_mm) + ofs_mm_center;
        let b_r_pix = (b_pix - cx_pix).abs();
        let b_r_mm = b_mm.abs();
        let b_theta = b_r_mm.atan2(camera_distance_mm); // for viewing only
                                                        // sample_data.push((i, b_pix, b_mm, b_r_pix, b_r_mm, b_theta));
        sample_data.push((b_r_pix, b_r_mm));
        pixs.push(b_r_pix);
        xs.push(b_r_pix / pixel_width * 2.0 * x_scale);
        ys.push(b_theta);
    }
    dbg!(&xs);
    dbg!(&ys);
    let frame_mm_to_angle = min_squares::<3, 9>(&xs, &ys);
    let angle_to_frame_mm = min_squares::<3, 9>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<4, 16>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<4, 16>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<6, 36>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<6, 36>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<10, 100>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<10, 100>(&ys, &xs);
    dbg!(frame_mm_to_angle);
    use CalcPoly;
    let mut e_sq = 0.;
    for i in 0..xs.len() {
        let nx = (&angle_to_frame_mm).calc(ys[i]);
        eprintln!(
            "{} {} {} : {} : {} v {} : {} : {}",
            i,
            xs[i],
            ys[i],
            nx,
            pixs[i],
            nx / x_scale / 2.0 * pixel_width,
            ((nx - xs[i]) * (nx - xs[i])).sqrt(),
            ((nx - xs[i]) * (nx - xs[i])).sqrt() / x_scale / 2.0 * pixel_width,
        );
        let e = ((nx - xs[i]) * (nx - xs[i])).sqrt() / x_scale / 2.0 * pixel_width;
        if e > 1.5 {
            e_sq += e;
        }
    }
    dbg!(e_sq);
    //     assert!(false);
    // sample_data.sort(cmp=lambda x,y:cmp(x[3],y[3])) # Sort by pixels, in case a plots is needed
}

//tp CalibrationData
pub struct CalibrationData {
    sensor: RectSensor,
    lens_from_frame: f64,
    image_from_lens: f64,
    /// For when the image was not quite parallel to camera
    angle_of_image: f64,
    /// Vec of image mm from centre to 2D point on sensor
    data: Vec<(f64, Point2D)>,
}
//ip CalibrationData
impl CalibrationData {
    //fp new
    fn new(
        sensor: RectSensor,
        lens_from_frame: f64,
        image_from_lens: f64,
        angle_of_image: f64,
    ) -> Self {
        let data = vec![];
        Self {
            sensor,
            lens_from_frame,
            image_from_lens,
            angle_of_image,
            data,
        }
    }
    //fp add_data
    fn add_data(&mut self, mm_image: f64, pt_sensor: Point2D) {
        self.data.push((mm_image, pt_sensor));
    }
    //fp tan_image
    fn tan_image(&self, dx: f64) -> f64 {
        let z = self.image_from_lens - dx * self.angle_of_image.sin();
        let x = dx * self.angle_of_image.cos();
        x / z
    }
    //fp tan_sensor
    fn tan_sensor(&self, pt: Point2D) -> f64 {
        let dx = self.sensor.px_abs_xy_to_px_rel_xy(pt)[0] * self.sensor.mm_single_pixel_width();
        // dbg!(pt, dx);
        dx / self.lens_from_frame
    }
    //fp analyze
    fn analyze(&self) -> (Vec<f64>, Vec<f64>, f64) {
        let mut ti = vec![];
        let mut ts = vec![];
        for (mm_image, pt_sensor) in self.data.iter() {
            ti.push(self.tan_image(*mm_image).abs());
            ts.push(self.tan_sensor(*pt_sensor).abs());
        }
        // dbg!(&ti);
        // dbg!(&ts);
        // let poly = min_squares::<8, 64>(&ti, &ts);
        // let poly = min_squares::<7, 49>(&ti, &ts);
        // let poly = min_squares::<6, 36>(&ti, &ts);
        let poly = min_squares::<5, 25>(&ti, &ts);
        let inv_poly = min_squares::<5, 25>(&ts, &ti);
        // let poly = min_squares::<4, 16>(&ti, &ts);
        // let inv_poly = min_squares::<4, 16>(&ts, &ti);
        dbg!(poly);
        let mut tot_e_sq = 0.;
        for (i, tan_image) in ti.iter().enumerate() {
            let tan_sensor = (&poly).calc(*tan_image);
            let px_rel_sensor =
                tan_sensor * self.lens_from_frame / self.sensor.mm_single_pixel_width();
            let diff = tan_sensor - ts[i];
            let diff_px = diff * self.lens_from_frame / self.sensor.mm_single_pixel_width();
            let e_sq = diff_px * diff_px;
            eprintln!(
                "{} {} {} : {} : {} {} : {} v {} : {} : {}",
                i,
                ti[i],
                ts[i],
                tan_sensor,
                self.data[i].0,
                self.data[i].1,
                self.sensor.px_centre()[0] + px_rel_sensor,
                self.sensor.px_centre()[0] - px_rel_sensor,
                diff_px,
                e_sq,
            );
            if e_sq > 0. {
                tot_e_sq += e_sq;
            }
        }
        dbg!(tot_e_sq);
        (poly.to_vec(), inv_poly.to_vec(), tot_e_sq)
    }

    //zz All done
}

//fp find_poly_for_canon_50mm
#[test]
fn find_poly_for_canon_50mm() {
    let sensor = RectSensor::new_35mm(6720, 4480);
    let mut calibration_data =
        CalibrationData::new(sensor, 50.0, 460.0 - 50.0, (1.83_f64).to_radians());

    // first bar is at -140mm, centre offset of +0.35mm (3368.0-3360)/(3368 - 3140)*10.0?
    const BARS_AT_50MM: &[usize] = &[
        246, 457, 675, 892, 1110, 1331, 1554, 1777, 2003, 2229, 2456, 2680, 2910, 3140, 3368, 3597,
        3825, 4057, 4287, 4513, 4745, 4973, 5202, 5430, 5660, 5884, 6111, 6336, 6560,
    ];

    for (i, px) in BARS_AT_50MM.iter().enumerate() {
        calibration_data.add_data((i as f64 - 14.0) * 10. + 0.35, [*px as f64, 0.].into());
    }
    let (_, _, tot_e_sq) = calibration_data.analyze();
    assert!(
        tot_e_sq < 60.0,
        "If all is working total error should be about 51.5"
    );
}

//fp find_poly_for_canon_50mm_at_short
#[test]
fn find_poly_for_canon_50mm_at_short() {
    let sensor = RectSensor::new_35mm(6720, 4480);
    let mut calibration_data =
        CalibrationData::new(sensor, 57.19, 460.0 - 57.19, (1.83_f64).to_radians());

    // first bar is at -120mm, offset of +0.68mm (3378.0-3360)/(3378 - 3325)*2.0?
    const BARS_AT_57_5_MM: &[usize] = &[
        245, 495, 750, 1007, 1264, 1523, 1785, 2049, 2312, 2577, 2844, 3111, 3378, 3645, 3914,
        4181, 4449, 4716, 4985, 5250, 5516, 5781, 6045, 6306, 6567,
    ];

    for (i, px) in BARS_AT_57_5_MM.iter().enumerate() {
        calibration_data.add_data((i as f64 - 12.0) * 10. + 0.68, [*px as f64, 0.].into());
    }
    let (_, _, tot_e_sq) = calibration_data.analyze();
    assert!(
        tot_e_sq < 40.0,
        "If all is working total error should be about 24.4"
    );
}
//fp compare_polys_for_canon_50mm
#[test]
fn compare_polys_for_canon_50mm() {
    let sensor = RectSensor::new_35mm(6720, 4480);

    let mut calibration_data_50mm =
        CalibrationData::new(sensor.clone(), 50.0, 460.0 - 50.0, (1.83_f64).to_radians());
    let mut calibration_data_57mm = CalibrationData::new(
        sensor.clone(),
        57.201,
        460.0 - 57.201,
        (1.83_f64).to_radians(),
    );

    // first bar is at -140mm, centre offset of +0.35mm (3368.0-3360)/(3368 - 3140)*10.0?
    const BARS_AT_50MM: &[usize] = &[
        246, 457, 675, 892, 1110, 1331, 1554, 1777, 2003, 2229, 2456, 2680, 2910, 3140, 3368, 3597,
        3825, 4057, 4287, 4513, 4745, 4973, 5202, 5430, 5660, 5884, 6111, 6336, 6560,
    ];

    for (i, px) in BARS_AT_50MM.iter().enumerate() {
        calibration_data_50mm.add_data((i as f64 - 14.0) * 10. + 0.35, [*px as f64, 0.].into());
    }

    // first bar is at -120mm, offset of +0.68mm (3378.0-3360)/(3378 - 3325)*2.0?
    const BARS_AT_57_5_MM: &[usize] = &[
        245, 495, 750, 1007, 1264, 1523, 1785, 2049, 2312, 2577, 2844, 3111, 3378, 3645, 3914,
        4181, 4449, 4716, 4985, 5250, 5516, 5781, 6045, 6306, 6567,
    ];

    for (i, px) in BARS_AT_57_5_MM.iter().enumerate() {
        calibration_data_57mm.add_data((i as f64 - 12.0) * 10. + 0.68, [*px as f64, 0.].into());
    }

    let (p50, _, _) = calibration_data_50mm.analyze();
    let (p57, _, _) = calibration_data_57mm.analyze();

    // The polynomial should be good up to about 20 degrees (half horizontal FOV)
    // which is 0.36 in tan() space
    // For diagonal 50mm lens FOV, about 23.4 degrees (half diagonal FOV that is)
    // which is 0.43 in tan() space
    let mut tot_e_sq = 0.;
    for i in 0..100 {
        let ti = i as f64 * 0.0032;
        let ts_50 = p50.calc(ti);
        let ts_57 = p57.calc(ti);
        let px_50 = ts_50 * 50.0 / sensor.mm_single_pixel_width();
        let px_57 = ts_57 * 50.0 / sensor.mm_single_pixel_width();
        let diff_px = px_57 - px_50;
        let e_sq = diff_px * diff_px;
        eprintln!(
            "{} {} : {} {} : {} {} : {} : {}",
            i, ti, ts_50, ts_50, px_50, px_57, diff_px, e_sq
        );
        tot_e_sq += e_sq;
    }
    assert!(
        tot_e_sq < 10.,
        "Total error should be < 100. but was {}",
        tot_e_sq
    );
}
