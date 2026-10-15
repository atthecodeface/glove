//a Imports
use thiserror::Error;

use crate::Point2D;

//a WordError
//tp WordError
#[derive(Error, Debug)]
pub enum WordError {
    #[error("{0}-coordinate with value {1} out of range 0-100")]
    CoordinateOutOfRange(char, u8),
    #[error("unknown string {1} for {0}-coordinate")]
    UnknownString(char, String),
    #[error("bad format for WordXy (must be <x word>.<y word>")]
    BadFormat,
}

pub trait IsWordXy {
    /// The number (for each array) of strings in the array
    const MAX_N: usize;
    /// Largest power of 2 that MAX_N is greater than
    ///
    /// The bottom MAX_N_2_PWR bits can be used directly for X, or for Y, for a
    /// mapping across the whole space.
    const MAX_N_2_PWR: usize;
    /// Coprime to MAX_N to make linear-to-XY mapping spread the X coordinate
    const X_MULT: usize;
    /// Coprime to MAX_N to make linear-to-XY mapping spread the Y coordinate
    const Y_MULT: usize;
    /// The length of all the strings
    const SLEN: usize;
    /// Array of MAX_N strings each of SLEN length
    const WORDS_X: &'static [&'static str];
    /// Array of MAX_N strings each of SLEN length
    const WORDS_Y: &'static [&'static str];
    fn xy_of_n(xy: usize) -> (u8, u8) {
        // Spread it around the space first
        let xy = xy * Self::X_MULT;
        // Now select X and Y using the bottom bits evenly,
        // and upper bits module MAX_N / 2^MAX_N_2_PWR
        let mut x = 0;
        let mut y = 0;
        if (xy & (1 << 0) != 0) && (Self::MAX_N_2_PWR > 0) {
            x += 1;
        }
        if (xy & (1 << 2) != 0) && (Self::MAX_N_2_PWR > 1) {
            x += 2;
        }
        if (xy & (1 << 4) != 0) && (Self::MAX_N_2_PWR > 2) {
            x += 4;
        }
        if (xy & (1 << 6) != 0) && (Self::MAX_N_2_PWR > 3) {
            x += 8;
        }
        if (xy & (1 << 8) != 0) && (Self::MAX_N_2_PWR > 5) {
            x += 16;
        }

        if (xy & (1 << 1) != 0) && (Self::MAX_N_2_PWR > 0) {
            y += 1;
        }
        if (xy & (1 << 3) != 0) && (Self::MAX_N_2_PWR > 1) {
            y += 2;
        }
        if (xy & (1 << 5) != 0) && (Self::MAX_N_2_PWR > 2) {
            y += 4;
        }
        if (xy & (1 << 7) != 0) && (Self::MAX_N_2_PWR > 3) {
            y += 8;
        }
        if (xy & (1 << 9) != 0) && (Self::MAX_N_2_PWR > 5) {
            y += 16;
        }

        let xy = xy >> (2 * Self::MAX_N_2_PWR);
        x += (xy % (Self::MAX_N >> Self::MAX_N_2_PWR)) << Self::MAX_N_2_PWR;
        y += (xy / (Self::MAX_N >> Self::MAX_N_2_PWR)) << Self::MAX_N_2_PWR;

        // Spread more evenly throughout the space
        x = ((x * Self::X_MULT) + Self::Y_MULT) % Self::MAX_N;
        y = (((y + x) * Self::Y_MULT) + Self::X_MULT) % Self::MAX_N;
        (x as u8, y as u8)
    }
}

pub struct MaybeWordXy<const N: u8, const M: u8>;

impl IsWordXy for MaybeWordXy<100, 5> {
    const MAX_N: usize = 100;
    const MAX_N_2_PWR: usize = 2;
    const X_MULT: usize = 37;
    const Y_MULT: usize = 73;

    const SLEN: usize = 5;
    const WORDS_X: &'static [&'static str] = &WORDS_X5;
    const WORDS_Y: &'static [&'static str] = &WORDS_Y5;
}

impl IsWordXy for MaybeWordXy<100, 6> {
    const MAX_N: usize = 100;
    const MAX_N_2_PWR: usize = 2;
    const X_MULT: usize = 37;
    const Y_MULT: usize = 73;
    const SLEN: usize = 6;
    const WORDS_X: &'static [&'static str] = &WORDS_X6;
    const WORDS_Y: &'static [&'static str] = &WORDS_Y6;
}

pub const WORDS_X5: [&str; 100] = [
    "adult", "audio", "black", "blood", "brief", "broad", "chain", "chief", "civil", "clear",
    "comic", "cover", "cross", "curve", "daily", "death", "early", "earth", "empty", "enemy",
    "equal", "extra", "field", "final", "first", "fixed", "fluid", "focus", "force", "fresh",
    "front", "fruit", "great", "green", "group", "heavy", "human", "inner", "known", "large",
    "later", "least", "legal", "level", "local", "logic", "magic", "major", "metal", "minor",
    "model", "moral", "movie", "named", "north", "noted", "novel", "ocean", "opera", "outer",
    "paper", "piano", "pilot", "place", "plant", "point", "power", "prior", "radio", "rapid",
    "ready", "right", "river", "royal", "rural", "sales", "short", "small", "solar", "solid",
    "south", "space", "spent", "split", "steam", "steel", "stock", "stone", "sugar", "total",
    "trade", "trial", "truth", "urban", "usual", "valid", "video", "voice", "wrong", "young",
];

//cp WORDS_Y5
pub const WORDS_Y5: [&str; 100] = [
    "agent", "album", "angle", "atoms", "award", "birds", "board", "boats", "books", "brain",
    "cases", "cells", "child", "claim", "class", "clock", "clubs", "coast", "color", "court",
    "crime", "dance", "drive", "drugs", "entry", "error", "event", "films", "fleet", "floor",
    "frame", "funds", "games", "goals", "goods", "hands", "heart", "house", "ideas", "image",
    "index", "issue", "items", "kinds", "label", "limit", "lines", "marks", "match", "media",
    "money", "music", "noise", "offer", "orbit", "pages", "parts", "phase", "piece", "pitch",
    "plays", "price", "proof", "raise", "rates", "roads", "roots", "rules", "scene", "seats",
    "sense", "ships", "shows", "signs", "sleep", "songs", "staff", "state", "study", "table",
    "tanks", "taxes", "tests", "title", "tools", "towns", "track", "trees", "union", "units",
    "value", "views", "visit", "votes", "walls", "waves", "weeks", "women", "works", "years",
];

//cp WORDS_X6
pub const WORDS_X6: [&str; 100] = [
    "access", "action", "agreed", "almost", "animal", "appear", "around", "author", "became",
    "behind", "better", "bodies", "bridge", "cannot", "career", "center", "choice", "cities",
    "closed", "county", "create", "degree", "design", "direct", "during", "effect", "either",
    "engine", "ensure", "events", "exists", "failed", "famous", "fields", "flight", "forces",
    "friend", "gained", "groups", "having", "helped", "houses", "images", "income", "island",
    "itself", "killed", "leader", "levels", "little", "longer", "making", "matter", "memory",
    "middle", "months", "mother", "moving", "nature", "needed", "number", "occurs", "office",
    "opened", "origin", "output", "passed", "period", "plants", "points", "powers", "public",
    "rather", "record", "refers", "remain", "return", "safety", "school", "season", "sector",
    "sexual", "should", "signal", "single", "sought", "speech", "spring", "stable", "strong",
    "supply", "system", "things", "toward", "troops", "unable", "useful", "volume", "weight",
    "within",
];

//cp WORDS_Y6
pub const WORDS_Y6: [&str; 100] = [
    "across", "actual", "allows", "always", "annual", "argued", "attack", "battle", "before",
    "belief", "beyond", "border", "called", "carbon", "caused", "change", "church", "claims",
    "common", "course", "damage", "demand", "device", "double", "easily", "effort", "energy",
    "enough", "entire", "except", "factor", "family", "female", "figure", "follow", "fourth",
    "future", "global", "growth", "health", "higher", "humans", "impact", "inside", "issues",
    "joined", "larger", "length", "likely", "living", "mainly", "market", "member", "method",
    "modern", "mostly", "motion", "nation", "nearly", "normal", "object", "offers", "oldest",
    "orders", "others", "oxygen", "people", "pieces", "played", "policy", "proved", "raised",
    "recent", "reduce", "region", "result", "rights", "saying", "search", "second", "served",
    "shared", "showed", "simple", "social", "source", "spread", "square", "states", "summer",
    "symbol", "theory", "though", "travel", "turned", "unique", "values", "wanted", "widely",
    "worked",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct WordXy<const N: u8, const M: u8>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    x: u8,
    y: u8,
}

impl<const N: u8, const M: u8> std::fmt::Display for WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}.{}",
            MaybeWordXy::<N, M>::WORDS_X[self.x as usize],
            MaybeWordXy::<N, M>::WORDS_Y[self.y as usize]
        )
    }
}

impl<const N: u8, const M: u8> std::convert::From<WordXy<N, M>> for (u8, u8)
where
    MaybeWordXy<N, M>: IsWordXy,
{
    fn from(xy: WordXy<N, M>) -> (u8, u8) {
        (xy.x, xy.y)
    }
}

impl<const N: u8, const M: u8> std::convert::From<(u8, u8)> for WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    fn from((x, y): (u8, u8)) -> WordXy<N, M> {
        Self::of_xy(x, y)
    }
}

impl<const N: u8, const M: u8> std::convert::From<(&(f64, f64, f64, f64), &Point2D)>
    for WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    fn from((bounds, pt): (&(f64, f64, f64, f64), &Point2D)) -> WordXy<N, M> {
        let x = (pt[0] - bounds.0) / (bounds.2 - bounds.0);
        let y = (pt[1] - bounds.1) / (bounds.3 - bounds.1);
        let x = (x.clamp(0., 0.9999) * (N as f64)).trunc() as u8;
        let y = (y.clamp(0., 0.9999) * (N as f64)).trunc() as u8;
        Self::of_xy(x, y)
    }
}

impl<const N: u8, const M: u8> std::convert::TryFrom<&str> for WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    type Error = WordError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        WordXy::<N, M>::parse_string(s)
    }
}

impl<const N: u8, const M: u8> std::str::FromStr for WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    type Err = WordError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        WordXy::<N, M>::parse_string(s)
    }
}

impl<const N: u8, const M: u8> WordXy<N, M>
where
    MaybeWordXy<N, M>: IsWordXy,
{
    #[track_caller]
    pub fn of_xy(x: u8, y: u8) -> Self {
        assert!(x < N, "X must be less than {N}");
        assert!(y < N, "Y must be less than {N}");
        Self { x, y }
    }

    fn parse_string<A: AsRef<str>>(s: A) -> Result<Self, WordError> {
        if let Some((x, y)) = s.as_ref().split_once('.') {
            let Some(x) = Self::find_x(x) else {
                return Err(WordError::UnknownString('x', x.to_owned()));
            };
            let Some(y) = Self::find_y(y) else {
                return Err(WordError::UnknownString('y', y.to_owned()));
            };
            Ok(Self::of_xy(x, y))
        } else {
            Err(WordError::BadFormat)
        }
    }

    fn find_s<A: AsRef<str>>(s: A, pile: &[&str]) -> Option<u8> {
        let s = s.as_ref();
        if s.len() != MaybeWordXy::<N, M>::SLEN {
            return None;
        }
        if !s.is_ascii() {
            return None;
        }
        let mut lc_s = [0_u8; 10];
        for (n, b) in s.as_bytes().iter().enumerate() {
            if b.is_ascii_uppercase() {
                lc_s[n] = *b | 0x20;
            } else {
                lc_s[n] = *b;
            }
        }
        let sel_lc_s = &lc_s[0..MaybeWordXy::<N, M>::SLEN];
        let f = |x_word: &&str| x_word.as_bytes().cmp(sel_lc_s);
        match pile.binary_search_by(f) {
            Ok(n) => Some(n as u8),
            _ => None,
        }
    }

    fn find_x<A: AsRef<str>>(s: A) -> Option<u8> {
        Self::find_s(s, MaybeWordXy::<N, M>::WORDS_X)
    }

    fn find_y<A: AsRef<str>>(s: A) -> Option<u8> {
        Self::find_s(s, MaybeWordXy::<N, M>::WORDS_Y)
    }

    /// Currently for test
    pub fn x(&self) -> u8 {
        self.x
    }
    pub fn y(&self) -> u8 {
        self.y
    }
    /// Get the 'nth' word pair, where the first N*N are distinct
    pub fn of_n(xy: usize) -> Self {
        MaybeWordXy::<N, M>::xy_of_n(xy).into()
    }
}
