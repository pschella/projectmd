//! Civil dates and the date expressions of GRAMMAR §5.1 (no date crate).

use serde::Serialize;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Civil {
    pub y: i32,
    pub m: u8,
    pub d: u8,
}

impl fmt::Display for Civil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

impl Civil {
    pub fn new(y: i32, m: u8, d: u8) -> Civil {
        Civil { y, m, d }
    }
    /// Days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        days_from_civil(self.y as i64, self.m as i64, self.d as i64)
    }
    pub fn from_days(z: i64) -> Civil {
        let z = z + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097);
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        Civil { y: y as i32, m: m as u8, d: d as u8 }
    }
    /// Add `n` calendar months, clamping the day to the target month's length.
    pub fn add_months(self, n: i64) -> Civil {
        let idx = self.y as i64 * 12 + (self.m as i64 - 1) + n;
        let y = idx.div_euclid(12) as i32;
        let m = (idx.rem_euclid(12) + 1) as u8;
        let d = self.d.min(days_in_month(y, m));
        Civil { y, m, d }
    }
}

#[cfg(test)]
mod month_tests {
    use super::Civil;
    #[test]
    fn add_months_calendar() {
        assert_eq!(Civil::new(2027, 1, 31).add_months(1), Civil::new(2027, 2, 28));
        assert_eq!(Civil::new(2028, 1, 31).add_months(1), Civil::new(2028, 2, 29));
        assert_eq!(Civil::new(2027, 11, 15).add_months(3), Civil::new(2028, 2, 15));
        assert_eq!(Civil::new(2027, 3, 31).add_months(-1), Civil::new(2027, 2, 28));
        assert_eq!(Civil::new(2026, 12, 1).add_months(12), Civil::new(2027, 12, 1));
    }
    #[test]
    fn duration_end_is_inclusive() {
        use crate::model::Duration;
        let d = |n, unit, days| Duration { n, unit, days, text: String::new() };
        assert_eq!(d(6, 'w', 42).end_from(Civil::new(2026, 11, 14)), Civil::new(2026, 12, 25));
        assert_eq!(d(1, 'd', 1).end_from(Civil::new(2026, 11, 14)), Civil::new(2026, 11, 14));
        assert_eq!(d(1, 'm', 30).end_from(Civil::new(2027, 1, 15)), Civil::new(2027, 2, 14));
    }
}

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateParse {
    /// Does not have the shape of a date expression.
    NotDate,
    /// Has the shape but is not a real date (E-DATE).
    Invalid,
    /// Valid; expanded start..end.
    Ok(Civil, Civil),
}

#[inline]
fn dig(b: &[u8]) -> Option<u32> {
    let mut v = 0u32;
    for &c in b {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v * 10 + (c - b'0') as u32;
    }
    Some(v)
}

/// Parse one date expression word (GRAMMAR §5.1).
pub fn parse_date(s: &str) -> DateParse {
    let b = s.as_bytes();
    if b.len() != 7 && b.len() != 10 || b[4] != b'-' {
        return DateParse::NotDate;
    }
    let y = match dig(&b[0..4]) {
        Some(v) => v as i32,
        None => return DateParse::NotDate,
    };
    match b.len() {
        10 => {
            if b[7] != b'-' {
                return DateParse::NotDate;
            }
            let (m, d) = match (dig(&b[5..7]), dig(&b[8..10])) {
                (Some(m), Some(d)) => (m, d),
                _ => return DateParse::NotDate,
            };
            if m < 1 || m > 12 || d < 1 || d > days_in_month(y, m as u8) as u32 {
                return DateParse::Invalid;
            }
            let c = Civil::new(y, m as u8, d as u8);
            DateParse::Ok(c, c)
        }
        7 => match b[5] {
            b'Q' => match b[6] {
                q @ b'1'..=b'4' => {
                    let q = (q - b'0') as u8;
                    let m1 = (q - 1) * 3 + 1;
                    let m3 = m1 + 2;
                    DateParse::Ok(Civil::new(y, m1, 1), Civil::new(y, m3, days_in_month(y, m3)))
                }
                _ => DateParse::NotDate,
            },
            _ => match dig(&b[5..7]) {
                Some(m) => {
                    if m < 1 || m > 12 {
                        DateParse::Invalid
                    } else {
                        DateParse::Ok(
                            Civil::new(y, m as u8, 1),
                            Civil::new(y, m as u8, days_in_month(y, m as u8)),
                        )
                    }
                }
                None => DateParse::NotDate,
            },
        },
        _ => DateParse::NotDate,
    }
}

/// Result of combining `A .. B`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeParse {
    NotRange,
    Invalid(&'static str),
    Ok(Civil, Civil),
}

pub fn range_of(a: &str, b: &str) -> RangeParse {
    let pa = parse_date(a);
    let pb = parse_date(b);
    match (pa, pb) {
        (DateParse::NotDate, _) | (_, DateParse::NotDate) => RangeParse::NotRange,
        (DateParse::Invalid, _) | (_, DateParse::Invalid) => RangeParse::Invalid("invalid date in range"),
        (DateParse::Ok(s, _), DateParse::Ok(_, e)) => {
            if e < s {
                RangeParse::Invalid("range ends before it starts")
            } else {
                RangeParse::Ok(s, e)
            }
        }
    }
}

/// A one-word range `D..D`: returns the two halves if both have date shape.
pub fn split_range_word(w: &str) -> Option<(&str, &str)> {
    let i = w.find("..")?;
    let (a, b) = (&w[..i], &w[i + 2..]);
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if parse_date(a) == DateParse::NotDate || parse_date(b) == DateParse::NotDate {
        return None;
    }
    Some((a, b))
}
