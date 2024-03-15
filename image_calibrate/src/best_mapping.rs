//a Imports

//a BestMapping
//tp BestMapping
/// A means for tracking the best mapping
#[derive(Debug, Clone)]
pub struct BestMapping<T: std::fmt::Display + std::fmt::Debug + Clone> {
    /// Asserted if the worst error should be used in evaluating error totals
    use_we: bool,
    /// The worst error
    we: f64,
    /// The total error
    te: f64,
    /// Associated data
    data: T,
}

//ip Copy for BestMapping<T>
impl<T> Copy for BestMapping<T> where T: std::fmt::Debug + std::fmt::Display + Copy {}

//ip BestMapping
impl<T> BestMapping<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone,
{
    //fp new
    /// Create a new best mapping
    pub fn new(use_we: bool, data: T) -> Self {
        Self {
            use_we,
            we: f64::MAX,
            te: f64::MAX,
            data,
        }
    }

    //ap we
    pub fn we(&self) -> f64 {
        self.we
    }

    //ap te
    pub fn te(&self) -> f64 {
        self.te
    }

    //ap data
    pub fn data(&self) -> &T {
        &self.data
    }

    //ap into_data
    pub fn into_data(self) -> T {
        self.data
    }

    //mp update_best
    /// Update the mapping with data if this is better
    pub fn update_best(&mut self, we: f64, te: f64, data: &T) -> bool {
        if self.use_we && we > self.we {
            return false;
        }
        if !self.use_we && te > self.te {
            return false;
        }
        self.we = we;
        self.te = te;
        self.data = data.clone();
        true
    }

    //cp best_of_both
    /// Pick the best of both
    pub fn best_of_both(self, other: Self) -> Self {
        let pick_self = if self.use_we {
            other.we > self.we
        } else {
            other.te > self.te
        };
        if pick_self {
            self
        } else {
            other
        }
    }

    //zz All done
}

//ip Display for Best
impl<T> std::fmt::Display for BestMapping<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "we: {:.4} te: {:.4} : {}", self.we, self.te, self.data,)
    }
}
