use serde::de::{self, Deserializer};
use serde::Deserialize;
use sqlx::types::time::Date;
use time::Month;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexDateInput {
	// Most common representations.
	Str(String),
	// `time::Date` can serialize as a 2-tuple [year, ordinal] depending on serde config.
	YearOrdinal(i32, u16),
	// Some clients may send [year, month, day].
	YearMonthDay(i32, u8, u8),
}

fn parse_yyyymmdd_digits(digits: &str) -> Option<Date> {
	if digits.len() < 8 {
		return None;
	}
	let y: i32 = digits.get(0..4)?.parse().ok()?;
	let m: u8 = digits.get(4..6)?.parse().ok()?;
	let d: u8 = digits.get(6..8)?.parse().ok()?;
	let month = Month::try_from(m).ok()?;
	Date::from_calendar_date(y, month, d).ok()
}

fn parse_flexible_date_str(s: &str) -> Option<Date> {
	let trimmed = s.trim();
	if trimmed.is_empty() {
		return None;
	}

	// Accept `YYYY-MM-DD` (strip non-digits).
	let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
	parse_yyyymmdd_digits(&digits)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
	D: Deserializer<'de>,
{
	let v = FlexDateInput::deserialize(deserializer)?;
	match v {
		FlexDateInput::Str(s) => parse_flexible_date_str(&s).ok_or_else(|| {
			de::Error::custom(
				"invalid date: expected YYYY-MM-DD or YYYYMMDD (or YYYYMMDDhhmmss)",
			)
		}),
		FlexDateInput::YearOrdinal(year, ordinal) => {
			Date::from_ordinal_date(year, u16::max(1, ordinal) as u16).map_err(
				|_| de::Error::custom("invalid date: expected [year, ordinal]"),
			)
		}
		FlexDateInput::YearMonthDay(year, month, day) => {
			let month = Month::try_from(month).map_err(|_| {
				de::Error::custom("invalid date: expected [year, month, day]")
			})?;
			Date::from_calendar_date(year, month, day).map_err(|_| {
				de::Error::custom("invalid date: expected [year, month, day]")
			})
		}
	}
}

pub fn deserialize_option_date<'de, D>(
	deserializer: D,
) -> Result<Option<Date>, D::Error>
where
	D: Deserializer<'de>,
{
	let opt = Option::<FlexDateInput>::deserialize(deserializer)?;
	let Some(v) = opt else { return Ok(None) };
	match v {
		FlexDateInput::Str(s) => Ok(parse_flexible_date_str(&s)),
		FlexDateInput::YearOrdinal(year, ordinal) => {
			Ok(Date::from_ordinal_date(year, u16::max(1, ordinal) as u16).ok())
		}
		FlexDateInput::YearMonthDay(year, month, day) => {
			let month = Month::try_from(month).ok();
			Ok(month.and_then(|m| Date::from_calendar_date(year, m, day).ok()))
		}
	}
}
