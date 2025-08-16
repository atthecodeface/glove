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

//a IsWordXy, MaybeWordXy, WORDS constatns
//tt IsWordXy
trait IsWordXy {
    const MAX_N: u8;
    const SLEN: usize;
    const WORDS_X: &'static [&'static str];
    const WORDS_Y: &'static [&'static str];
}

//ti MaybeWordXy
struct MaybeWordXy<const N: u8>;

//ii IsWordXy for MaybeWordXy<50>
impl IsWordXy for MaybeWordXy<50> {
    const MAX_N: u8 = 50;
    const SLEN: usize = 5;
    const WORDS_X: &'static [&'static str] = &WORDS_X5;
    const WORDS_Y: &'static [&'static str] = &WORDS_Y5;
}

//ii IsWordXy for MaybeWordXy<100>
impl IsWordXy for MaybeWordXy<100> {
    const MAX_N: u8 = 100;
    const SLEN: usize = 6;
    const WORDS_X: &'static [&'static str] = &WORDS_X6;
    const WORDS_Y: &'static [&'static str] = &WORDS_Y6;
}

//a Constants for the words
//cp WORDS_X5
pub const WORDS_X5: [&str; 50] = [
    "about", "after", "album", "areas", "based", "being", "birth", "board", "cause", "civil",
    "clear", "court", "early", "every", "field", "final", "focus", "front", "given", "hands",
    "human", "issue", "large", "local", "means", "metal", "music", "never", "occur", "parts",
    "point", "prior", "radio", "rules", "serve", "short", "songs", "speed", "still", "taken",
    "their", "three", "title", "trade", "units", "users", "video", "which", "works", "wrote",
];

//cp WORDS_Y5
pub const WORDS_Y5: [&str; 50] = [
    "added", "again", "among", "avoid", "began", "below", "blood", "built", "cells", "class",
    "color", "death", "equal", "exist", "films", "first", "force", "fully", "group", "heavy",
    "image", "known", "least", "major", "media", "model", "needs", "noted", "other", "place",
    "power", "quite", "right", "scene", "ships", "since", "south", "state", "study", "terms",
    "third", "times", "today", "under", "until", "value", "where", "women", "would", "young",
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

//a WordXy
//tp WordXy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct WordXy<const N: u8>
where
    MaybeWordXy<N>: IsWordXy,
{
    x: u8,
    y: u8,
}

//ip Display for WordXy
impl<const N: u8> std::fmt::Display for WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}.{}",
            MaybeWordXy::<N>::WORDS_X[self.x as usize],
            MaybeWordXy::<N>::WORDS_Y[self.y as usize]
        )
    }
}

//ip From<WordXy> for (u8, u8)
impl<const N: u8> std::convert::From<WordXy<N>> for (u8, u8)
where
    MaybeWordXy<N>: IsWordXy,
{
    fn from(xy: WordXy<N>) -> (u8, u8) {
        (xy.x, xy.y)
    }
}

//ip From<(u8, u8)> for WordXy
impl<const N: u8> std::convert::From<(u8, u8)> for WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    fn from((x, y): (u8, u8)) -> WordXy<N> {
        Self::of_xy(x, y)
    }
}

//ip From<(&(f64, f64, f64, f64), &Point2D)> for WordXy
impl<const N: u8> std::convert::From<(&(f64, f64, f64, f64), &Point2D)> for WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    fn from((bounds, pt): (&(f64, f64, f64, f64), &Point2D)) -> WordXy<N> {
        let x = (pt[0] - bounds.0) / (bounds.2 - bounds.0);
        let y = (pt[1] - bounds.1) / (bounds.3 - bounds.1);
        let x = (x.clamp(0., 0.9999) * (N as f64)).trunc() as u8;
        let y = (y.clamp(0., 0.9999) * (N as f64)).trunc() as u8;
        Self::of_xy(x, y)
    }
}

//ip TryFrom for WordXy
impl<'a, const N: u8> std::convert::TryFrom<&'a str> for WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    type Error = WordError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        WordXy::<N>::parse_string(s)
    }
}

//ip FromStr for WordXy
impl<const N: u8> std::str::FromStr for WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    type Err = WordError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        WordXy::<N>::parse_string(s)
    }
}

//ip WordXy
impl<const N: u8> WordXy<N>
where
    MaybeWordXy<N>: IsWordXy,
{
    //cp of_xy
    #[track_caller]
    pub fn of_xy(x: u8, y: u8) -> Self {
        assert!(x < N, "X must be less than {N}");
        assert!(y < N, "Y must be less than {N}");
        Self { x, y }
    }

    //fi parse_string
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

    //fi find_s
    fn find_s<A: AsRef<str>>(s: A, pile: &[&str]) -> Option<u8> {
        let s = s.as_ref();
        if s.len() != MaybeWordXy::<N>::SLEN {
            return None;
        }
        if !s.is_ascii() {
            return None;
        }
        let mut lc_s = [0_u8; 10];
        for (n, b) in s.as_bytes().iter().enumerate() {
            if (b'A'..=b'Z').contains(&b) {
                lc_s[n] = *b | 0x20;
            } else {
                lc_s[n] = *b;
            }
        }
        let sel_lc_s = &lc_s[0..MaybeWordXy::<N>::SLEN];
        let f = |x_word: &&str| x_word.as_bytes().cmp(&sel_lc_s);
        match pile.binary_search_by(f) {
            Ok(n) => Some(n as u8),
            _ => None,
        }
    }

    //fi find_x
    fn find_x<A: AsRef<str>>(s: A) -> Option<u8> {
        Self::find_s(s, &MaybeWordXy::<N>::WORDS_X)
    }

    //fi find_y
    fn find_y<A: AsRef<str>>(s: A) -> Option<u8> {
        Self::find_s(s, &MaybeWordXy::<N>::WORDS_Y)
    }

    //zz All done
}

//a Tests
#[test]
fn test_word_xy_0() -> Result<(), Box<dyn std::error::Error>> {
    for (test_xy, xy_value) in [
        ("about.added", (0_u8, 0_u8)),
        ("after.again", (1_u8, 1_u8)),
        ("album.among", (2_u8, 2_u8)),
    ] {
        eprintln!("Test {test_xy}");
        let xy: WordXy<50> = test_xy.parse()?;
        assert_eq!(test_xy, &xy.to_string());
        assert_eq!(xy_value, xy.into());
    }
    Ok(())
}
