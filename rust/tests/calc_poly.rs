//a Modules
// use glove::calibrate::Projection;
use glove::calibrate::{min_squares, min_squares_dyn};
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
    let _poly_degree = 5;
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
        let _b_mm = (b_bar_num - 0.5) * bar_width_mm + ofs_mm_center;
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
        let dy = self.sensor.px_abs_xy_to_px_rel_xy(pt)[1] * self.sensor.mm_single_pixel_height();
        let d = (dx * dx + dy * dy).sqrt();
        // dbg!(pt, dx);
        d / self.lens_from_frame
    }
    //fp extract_tan_map_data
    fn extract_tan_map_data(&self) -> (Vec<f64>, Vec<f64>) {
        let mut ti = vec![];
        let mut ts = vec![];
        for (mm_image, pt_sensor) in self.data.iter() {
            ti.push(self.tan_image(*mm_image).abs());
            ts.push(self.tan_sensor(*pt_sensor).abs());
        }
        (ti, ts)
    }
}

//tp TanMap
pub struct TanMap {
    sensor: RectSensor,
    /// tan(image), tan(sensor)
    data: Vec<(f64, f64)>,
    /// Image-to-sensor tan-space map
    pub its_poly: Vec<f64>,
    /// Sensor-to-Image tan-space map
    pub sti_poly: Vec<f64>,
}
//ip TanMap
impl TanMap {
    //fp new
    fn new(sensor: RectSensor) -> Self {
        Self {
            sensor,
            data: vec![],
            its_poly: vec![],
            sti_poly: vec![],
        }
    }
    //fp add_calibration_data
    fn add_calibration_data(&mut self, calibration_data: &CalibrationData) {
        let (ti, ts) = calibration_data.extract_tan_map_data();
        for (ti, ts) in ti.into_iter().zip(ts.into_iter()) {
            self.data.push((ti, ts));
        }
    }

    //fp sort_data
    pub fn sort_data(&mut self) {
        self.data.sort_by(|a, b| a.0.partial_cmp(&(b.0)).unwrap());
    }

    //fp analyze
    fn analyze(&mut self, poly_degree: usize) {
        let mut ti = vec![];
        let mut ts = vec![];
        for (tan_image, tan_sensor) in self.data.iter() {
            ti.push(*tan_image);
            ts.push(*tan_sensor);
        }
        // dbg!(&ti);
        // dbg!(&ts);
        let its_poly = min_squares_dyn(poly_degree, &ti, &ts);
        let sti_poly = min_squares_dyn(poly_degree, &ts, &ti);
        self.its_poly = its_poly.to_vec();
        self.sti_poly = sti_poly.to_vec();
    }

    //fp replace_polys
    fn replace_polys(&mut self, its_poly: &[f64], sti_poly: &[f64]) {
        self.its_poly = its_poly.to_vec();
        self.sti_poly = sti_poly.to_vec();
    }

    //fp map_tan_image
    #[inline]
    pub fn map_tan_image(&self, tan_image: f64) -> f64 {
        self.its_poly.calc(tan_image)
    }

    //fp map_tan_sensor
    #[inline]
    pub fn map_tan_sensor(&self, tan_sensor: f64) -> f64 {
        self.sti_poly.calc(tan_sensor)
    }

    //fp debug
    fn debug(&self, lens_from_frame: f64) -> f64 {
        let mut tot_e_sq = 0.;
        for (i, (tan_image, tan_sensor)) in self.data.iter().enumerate() {
            let calc_tan_sensor = self.map_tan_image(*tan_image);
            let px_rel_sensor =
                calc_tan_sensor * lens_from_frame / self.sensor.mm_single_pixel_width();
            let diff = calc_tan_sensor - tan_sensor;
            let diff_px = diff * lens_from_frame / self.sensor.mm_single_pixel_width();
            let e_sq = diff_px * diff_px;
            eprintln!(
                "{} {} {} : {} : {} v {} : {} : {}",
                i,
                tan_image,
                tan_sensor,
                calc_tan_sensor,
                self.sensor.px_centre()[0] + px_rel_sensor,
                self.sensor.px_centre()[0] - px_rel_sensor,
                diff_px,
                e_sq,
            );
            if e_sq > 0. {
                tot_e_sq += e_sq;
            }
        }
        tot_e_sq
    }

    //zz All done
}

//fp find_poly_for_canon_50mm
#[test]
fn find_poly_for_canon_50mm() {
    let focal_length = 50.0;
    // let focal_length = 49.77;
    let sensor = RectSensor::new_35mm(6720, 4480);
    let mut calibration_data = CalibrationData::new(
        sensor.clone(),
        focal_length,
        460.0 - focal_length,
        (1.83_f64).to_radians(),
    );

    // first bar is at -140mm, centre offset of +0.35mm (3368.0-3360)/(3368 - 3140)*10.0?
    const BARS_AT_50MM: &[usize] = &[
        246, 457, 675, 892, 1110, 1331, 1554, 1777, 2003, 2229, 2456, 2680, 2910, 3140, 3368, 3597,
        3825, 4057, 4287, 4513, 4745, 4973, 5202, 5430, 5660, 5884, 6111, 6336, 6560,
    ];

    for (i, px) in BARS_AT_50MM.iter().enumerate() {
        calibration_data.add_data((i as f64 - 14.0) * 10. + 0.35, [*px as f64, 0.].into());
    }
    let mut tan_map = TanMap::new(sensor.clone());
    tan_map.add_calibration_data(&calibration_data);
    tan_map.analyze(5);
    let tot_e_sq = tan_map.debug(focal_length);
    assert!(
        tot_e_sq < 52.0,
        "If all is working total error should be about 51.5 was {}",
        tot_e_sq
    );
}

//fp find_poly_for_canon_50mm_y
#[test]
fn find_poly_for_canon_50mm_y() {
    let focal_length = 50.0;
    // let focal_length = 49.77;
    let sensor = RectSensor::new_35mm(6720, 4480);
    let mut calibration_data = CalibrationData::new(
        sensor.clone(),
        focal_length,
        460.0 - focal_length,
        (0.0_f64).to_radians(), // vertical door (and vertical camera?)
    );

    // first bar is at -90mm, centre offset of +1.6mm 10.0+(2240.0-2434)/(2434 - 2203)*10.0?
    const BARS_AT_50MM: &[usize] = &[
        (171 + 132) / 2,
        (363 + 395) / 2,
        (588 + 623) / 2,
        (813 + 845) / 2,
        (1040 + 1076) / 2,
        (1269 + 1302) / 2,
        (1500 + 1539) / 2,
        (1727 + 1763) / 2,
        (1958 + 1994) / 2, // 1976
        (2189 + 2217) / 2, // = 2203, at about 1.6mm
        (2417 + 2451) / 2, // = 2434
        (2645 + 2676) / 2,
        (2871 + 2907) / 2,
        (3101 + 3132) / 2,
        (3360 + 3330) / 2,
        (3557 + 3591) / 2,
        (3786 + 3818) / 2,
        (4010 + 4041) / 2,
        (4230 + 4265) / 2,
    ];

    for (i, px) in BARS_AT_50MM.iter().enumerate() {
        calibration_data.add_data((i as f64 - 9.0) * 10. - 1.7, [0., *px as f64].into());
    }
    let mut tan_map = TanMap::new(sensor.clone());
    tan_map.add_calibration_data(&calibration_data);
    tan_map.analyze(5);
    let tot_e_sq = tan_map.debug(focal_length);
    assert!(
        tot_e_sq < 10.0,
        "If all is working total error should be about 8.1 (!) was {}",
        tot_e_sq
    );
}

//fp find_poly_for_canon_50mm_at_short
#[test]
fn find_poly_for_canon_50mm_at_short() {
    let sensor = RectSensor::new_35mm(6720, 4480);
    let mut calibration_data = CalibrationData::new(
        sensor.clone(),
        57.19,
        460.0 - 57.19,
        (1.83_f64).to_radians(),
    );

    // first bar is at -120mm, offset of +0.68mm (3378.0-3360)/(3378 - 3325)*2.0?
    const BARS_AT_57_5_MM: &[usize] = &[
        245, 495, 750, 1007, 1264, 1523, 1785, 2049, 2312, 2577, 2844, 3111, 3378, 3645, 3914,
        4181, 4449, 4716, 4985, 5250, 5516, 5781, 6045, 6306, 6567,
    ];

    for (i, px) in BARS_AT_57_5_MM.iter().enumerate() {
        calibration_data.add_data((i as f64 - 12.0) * 10. + 0.68, [*px as f64, 0.].into());
    }
    let mut tan_map = TanMap::new(sensor.clone());
    tan_map.add_calibration_data(&calibration_data);
    tan_map.analyze(5);
    let tot_e_sq = tan_map.debug(57.19);
    assert!(
        tot_e_sq < 40.0,
        "If all is working total error should be about 24.4"
    );
}
//fp compare_polys_for_canon_50mm
#[test]
fn compare_polys_for_canon_50mm() {
    let do_sort = true;
    let sensor = RectSensor::new_35mm(6720, 4480);
    // let mm_closeup = 57.212;
    // let focal_length = 50.0;
    let focal_length = 49.77;
    let mm_closeup = 57.0;
    let mut calibration_data_50mm = CalibrationData::new(
        sensor.clone(),
        focal_length,
        460.0 - focal_length,
        (1.83_f64).to_radians(),
    );
    let mut calibration_data_57mm = CalibrationData::new(
        sensor.clone(),
        mm_closeup,
        460.0 - mm_closeup,
        (1.83_f64).to_radians(),
    );

    // first bar is at -140mm, centre offset of +0.35mm (3368.0-3360)/(3368 - 3140)*10.0?
    const BARS_AT_50MM: &[usize] = &[
        246, 457, 675, 892, 1110, 1331, 1554, 1777, 2003, 2229, 2456, 2680, 2910, 3140, 3368, 3597,
        3825, 4057, 4287, 4513, 4745, 4973, 5202, 5430, 5660, 5884, 6111, 6336, 6560,
    ];

    for (i, px) in BARS_AT_50MM.iter().enumerate() {
        calibration_data_50mm.add_data((i as f64 - 14.0) * 10. + 0.33, [*px as f64, 0.].into());
    }

    // first bar is at -120mm, offset of +0.68mm (3378.0-3360)/(3378 - 3325)*2.0?
    const BARS_AT_57_5_MM: &[usize] = &[
        245, 495, 750, 1007, 1264, 1523, 1785, 2049, 2312, 2577, 2844, 3111, 3378, 3645, 3914,
        4181, 4449, 4716, 4985, 5250, 5516, 5781, 6045, 6306, 6567,
    ];

    for (i, px) in BARS_AT_57_5_MM.iter().enumerate() {
        calibration_data_57mm.add_data((i as f64 - 12.0) * 10. + 0.65, [*px as f64, 0.].into());
    }

    let mut tan_map_50 = TanMap::new(sensor.clone());
    tan_map_50.add_calibration_data(&calibration_data_50mm);
    tan_map_50.sort_data();
    tan_map_50.analyze(5);

    let mut tan_map_57 = TanMap::new(sensor.clone()); // use 50.0 to compare with 50 data
    tan_map_57.add_calibration_data(&calibration_data_57mm);
    tan_map_57.sort_data();
    tan_map_57.analyze(5);

    let mut tan_map = TanMap::new(sensor.clone());
    tan_map.add_calibration_data(&calibration_data_50mm);
    tan_map.add_calibration_data(&calibration_data_57mm);
    if do_sort {
        tan_map.sort_data();
    }
    tan_map.analyze(6);
    let tot_e_sq = tan_map.debug(focal_length);
    dbg!(tot_e_sq);

    eprintln!("\nUsing combined poly for focus-at-infinity data");
    tan_map_50.replace_polys(&tan_map.its_poly, &tan_map.sti_poly);
    let tot_e_sq_50 = tan_map_50.debug(focal_length);
    dbg!(tot_e_sq_50);

    eprintln!("\nUsing combined poly for focus-closeup data");
    tan_map_57.replace_polys(&tan_map.its_poly, &tan_map.sti_poly);
    let tot_e_sq_57 = tan_map_57.debug(mm_closeup);
    dbg!(tot_e_sq_57);

    eprintln!("Tot err {} : {} : {}", tot_e_sq, tot_e_sq_50, tot_e_sq_57,);
    // assert!(false);

    // The polynomial should be good up to about 20 degrees (half horizontal FOV)
    // which is 0.36 in tan() space
    // For diagonal 50mm lens FOV, about 23.4 degrees (half diagonal FOV that is)
    // which is 0.43 in tan() space
}
