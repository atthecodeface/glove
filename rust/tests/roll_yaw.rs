//a Modules
use geo_nd::vector;
use glove::calibrate::{Point2D, RollYaw, TanXTanY};

//a Tests
#[test]
fn test0() {
    fn dist(p: TanXTanY, q: TanXTanY) -> f64 {
        let p: Point2D = p.into();
        let q: Point2D = q.into();
        vector::length((p - q).as_ref())
    }
    fn check_point(p: TanXTanY) {
        //       let p = vector::normalize(*(p.as_ref())).into();
        let ry = RollYaw::from_txty(p);
        let p2 = ry.to_txty();
        eprintln!("Checking point {} -> {} -> {}", p, ry, p2);
        assert!(dist(p, p2) < 1.0E4, "Difference too great");
    }
    for p in [[0., 1.], [1., 1.], [0., -1.], [-1., -1.]] {
        check_point(p.into());
    }
    for i in -100..100 {
        for j in -100..100 {
            check_point([(i as f64) * 0.1, (j as f64) * 0.1].into());
        }
    }
}
