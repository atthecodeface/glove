//a Imports
use serde::Serialize;
use std::collections::HashSet;

use crate::image::{DynamicImage, GenericImageView, Image};
use crate::Color;

//a Regions
//tp Region
/// This structure summarizes a region of an Image that is (for
/// example) of a single color, with the ability to determine the
/// centre-of-gravity and some idea of spread (size of the region)
///
/// Regions can be merged if they adjoin and are of the same color
#[derive(Debug, Default, Clone, Serialize)]
pub struct Region {
    /// Number of pixels
    n: usize,
    /// Sum of X coords
    x_sum: usize,
    /// Sum of Y coords
    y_sum: usize,
    /// Sum of square of X coords
    x2_sum: usize,
    /// Sum of square of Y coords
    y2_sum: usize,
    /// Color
    color: Color,
}

//ip Region
impl Region {
    //ap color
    /// Get the color of the region
    pub fn color(&self) -> Color {
        self.color
    }

    //bp with_color
    /// Build the region with a new color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    //cp of_color
    /// Construct an empty region with a color
    pub fn of_color(color: Color) -> Self {
        Self::default().with_color(color)
    }

    //mp is_of_color
    /// Return true if the region is of the color
    pub fn is_of_color(&self, color: &Color) -> bool {
        self.color.color_eq(color)
    }

    //ap cog
    /// Return the centre of gravity of the region
    ///
    /// An empty region is a (0.0, 0.0) centre-of-gravity
    pub fn cog(&self) -> (f64, f64) {
        (
            (self.x_sum as f64) / (self.n as f64),
            (self.y_sum as f64) / (self.n as f64),
        )
    }

    //ap spreads
    /// Return an estimate of the spread
    ///
    /// This is the standard deviation in the X and Y
    pub fn spreads(&self) -> (f64, f64) {
        let ex = (self.x_sum as f64) / (self.n as f64);
        let ex2 = (self.x2_sum as f64) / (self.n as f64);
        let vx = ex2 - ex * ex;

        let ey = (self.y_sum as f64) / (self.n as f64);
        let ey2 = (self.y2_sum as f64) / (self.n as f64);
        let vy = ey2 - ey * ey;

        let sd_x = vx.sqrt();
        let sd_y = vy.sqrt();
        (sd_x, sd_y)
    }

    //ap spread
    /// Return an estimate of the spread
    ///
    /// This is related to the standard deviation
    pub fn spread(&self) -> f64 {
        let ex = (self.x_sum as f64) / (self.n as f64);
        let ex2 = (self.x2_sum as f64) / (self.n as f64);
        let vx = ex2 - ex * ex;

        let ey = (self.y_sum as f64) / (self.n as f64);
        let ey2 = (self.y2_sum as f64) / (self.n as f64);
        let vy = ey2 - ey * ey;
        let v = vx + vy;
        v.sqrt()
    }

    //mp add_pixel
    /// Add a pixel to the region
    pub fn add_pixel(&mut self, x: usize, y: usize) {
        self.n += 1;
        self.x_sum += x;
        self.x2_sum += x * x;
        self.y_sum += y;
        self.y2_sum += y * y;
    }

    //mp merge
    /// Merge another region into this one
    pub fn merge(&mut self, other: &Self) {
        self.n += other.n;
        self.x_sum += other.x_sum;
        self.x2_sum += other.x2_sum;
        self.y_sum += other.y_sum;
        self.y2_sum += other.y2_sum;
    }

    //cp scan_image_to_regions
    fn scan_image_to_regions<F>(
        img: &DynamicImage,
        is_region: &F,
    ) -> (Vec<Region>, Vec<(usize, usize)>)
    where
        F: Fn(Color) -> bool,
    {
        let (xsz, ysz) = img.dimensions();
        let xsz = xsz as usize;
        let ysz = ysz as usize;
        let mut regions: Vec<Region> = vec![];
        let mut regions_to_merge: Vec<(usize, usize)> = vec![];
        let mut regions_x: Vec<Option<usize>> = vec![None; xsz];
        for y in 0..ysz {
            let regions_py = regions_x;
            regions_x = vec![None; xsz];
            for x in 0..xsz {
                let c = img.get(x as u32, y as u32);
                if !is_region(c) {
                    regions_x[x] = None;
                    continue;
                }
                let mut region = None;
                if x > 0
                    && regions_x[x - 1].is_some()
                    && regions[regions_x[x - 1].unwrap()].is_of_color(&c)
                {
                    region = regions_x[x - 1];
                }
                region = {
                    if let Some(py_region) = regions_py[x] {
                        if regions[py_region].is_of_color(&c) {
                            if let Some(region) = region {
                                if py_region != region {
                                    regions_to_merge.push((py_region, region));
                                }
                                Some(region)
                            } else {
                                Some(py_region)
                            }
                        } else {
                            region
                        }
                    } else {
                        region
                    }
                };
                let region = {
                    if let Some(region) = region {
                        region
                    } else {
                        let n = regions.len();
                        regions.push(Region::of_color(c));
                        n
                    }
                };
                regions[region].add_pixel(x, y);
                regions_x[x] = Some(region);
            }
        }
        (regions, regions_to_merge)
    }

    fn determine_regions_that_adjoin(
        regions: &[Region],
        regions_to_merge: &[(usize, usize)],
    ) -> Vec<HashSet<usize>> {
        let mut regions_that_adjoin: Vec<HashSet<usize>> = vec![];
        let mut adjoining_index_of_region = vec![];
        for i in 0..regions.len() {
            regions_that_adjoin.push(HashSet::new());
            regions_that_adjoin[i].insert(i);
            adjoining_index_of_region.push(i);
        }
        loop {
            let mut changed = false;
            for (r0, r1) in regions_to_merge {
                let mr0 = adjoining_index_of_region[*r0];
                let mr1 = adjoining_index_of_region[*r1];
                if mr0 == mr1 {
                    continue;
                }
                if mr0 < mr1 {
                    let merge_in = std::mem::take(&mut regions_that_adjoin[mr1]);
                    for r in merge_in {
                        adjoining_index_of_region[r] = mr0;
                        regions_that_adjoin[mr0].insert(r);
                    }
                } else {
                    let merge_in = std::mem::take(&mut regions_that_adjoin[mr0]);
                    for r in merge_in {
                        adjoining_index_of_region[r] = mr1;
                        regions_that_adjoin[mr1].insert(r);
                    }
                }
                changed = true;
            }
            if !changed {
                break;
            }
        }
        regions_that_adjoin
    }

    //fp regions_of_image
    pub fn regions_of_image<F>(img: &DynamicImage, is_region: &F) -> Vec<Region>
    where
        F: Fn(Color) -> bool,
    {
        let (regions, regions_to_merge) = Region::scan_image_to_regions(img, is_region);
        let regions_that_adjoin =
            Region::determine_regions_that_adjoin(&regions, &regions_to_merge);
        let mut merged_regions = vec![];
        for regions_to_merge in regions_that_adjoin {
            let mut mr = Region::default();
            for (i, r) in regions_to_merge.into_iter().enumerate() {
                if i == 0 {
                    mr = mr.with_color(regions[r].color());
                }
                mr.merge(&regions[r]);
            }
            merged_regions.push(mr);
        }
        merged_regions
    }

    //zz All done
}
