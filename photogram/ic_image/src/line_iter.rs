//tp LineIter
/// starts at (posn + .0, other + .error)
///
/// Next is either at (posn+1 + .0, other   + .(error+add_error))
///             or at (posn+1 + .0, other+1 + .(error+add_error-sub_error))
#[derive(Debug)]
pub struct LineIter {
    /// Current position - complete when posn > end, incremented on each tick
    posn: i32,
    /// End position (must be on-screen)
    end: u32,
    /// Other coordinate - incremented on each tick when 'error' overflows
    other: i32,
    /// Current error - add 'add_error' per tick; when this exceeds 1<<24, reduce by sub_error
    error: u32,
    /// Additional error per tick (< 1<<24)
    add_error: u32,
    /// Subtraction error when 'error' overflows (1<<24); must exceed 'add_error'
    sub_error: u32,
    /// Whether to increment or decrement 'other' on each tick
    incr_other: bool,
    /// Kind of line = posn/other is X/Y or Y/X
    posn_is_x: bool,
}

//ii LineIter
impl LineIter {
    //cp new
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Option<Self> {
        // eprintln!("{x0} {y0}   {x1} {y1}");
        if x0 < 0 && x1 < 0 {
            return None;
        }
        if y0 < 0 && y1 < 0 {
            return None;
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let mut s = {
            if dx.abs() > dy.abs() {
                if x0 <= x1 {
                    Self {
                        posn: x0,
                        other: y0,
                        end: x1.unsigned_abs(),
                        error: 1 << 24,
                        add_error: dy.unsigned_abs(),
                        sub_error: dx.unsigned_abs(),
                        incr_other: dy > 0,
                        posn_is_x: true,
                    }
                } else {
                    Self {
                        posn: x1,
                        other: y1,
                        end: x0.unsigned_abs(),
                        error: 1 << 24,
                        add_error: dy.unsigned_abs(),
                        sub_error: dx.unsigned_abs(),
                        incr_other: dy < 0,
                        posn_is_x: true,
                    }
                }
            } else if y0 <= y1 {
                Self {
                    posn: y0,
                    other: x0,
                    end: y1.unsigned_abs(),
                    error: 1 << 24,
                    add_error: dx.unsigned_abs(),
                    sub_error: dy.unsigned_abs(),
                    incr_other: dx > 0,
                    posn_is_x: false,
                }
            } else {
                Self {
                    posn: y1,
                    other: x1,
                    end: y0.unsigned_abs(),
                    error: 1 << 24,
                    add_error: dx.unsigned_abs(),
                    sub_error: dy.unsigned_abs(),
                    incr_other: dx < 0,
                    posn_is_x: false,
                }
            }
        };
        if s.posn < -1000 {
            return None;
        }
        // eprintln!("{s:?}");
        s.tick_until_posn_nonneg();
        if s.has_finished() {
            None
        } else {
            Some(s)
        }
    }

    //ai has_finished
    fn has_finished(&self) -> bool {
        (self.posn as u32) > self.end
    }

    //mi tick
    fn tick(&mut self) -> bool {
        self.posn += 1;
        self.error += self.add_error;
        if self.error > (1 << 24) {
            self.error -= self.sub_error;
            if self.incr_other {
                self.other += 1;
            } else {
                self.other -= 1;
            }
        }
        self.has_finished()
    }

    //mi tick
    fn tick_until_posn_nonneg(&mut self) {
        if self.posn < 0 {
            while self.tick() {
                if self.posn >= 0 {
                    break;
                }
            }
        }
        if self.other < 0 {
            if !self.incr_other {
                self.posn = (self.end + 1) as i32;
            } else {
                while self.tick() {
                    if self.other >= 0 {
                        break;
                    }
                }
            }
        }
    }
}

//ii Iterator for LineIter
impl Iterator for LineIter {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<(u32, u32)> {
        if self.has_finished() {
            None
        } else {
            let (p, o) = (self.posn as u32, self.other as u32);
            self.tick();
            if self.posn_is_x {
                Some((p, o))
            } else {
                Some((o, p))
            }
        }
    }
}
