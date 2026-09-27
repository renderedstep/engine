//! Times as the database holds them: `YYYY-MM-DD HH:MM:SS`, UTC, with six
//! digits of fraction when there is any. The engine reads a time as whole
//! seconds since the Unix epoch, as the Ruby engine's records do.

use std::time::{SystemTime, UNIX_EPOCH};

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of a count of days since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Whole seconds since the epoch of a stored time, the fraction dropped.
/// None for text that is not a time.
pub fn parse(text: &str) -> Option<i64> {
    let text = text.trim();
    let (date, time) = text.split_once([' ', 'T'])?;
    let mut date = date.splitn(3, '-');
    let year: i64 = date.next()?.parse().ok()?;
    let month: i64 = date.next()?.parse().ok()?;
    let day: i64 = date.next()?.parse().ok()?;
    let time = time.trim_end_matches('Z');
    let time = time.split(['.', '+']).next()?;
    let mut clock = time.splitn(3, ':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next().unwrap_or("0").parse().ok()?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

/// A time in whole seconds, as the database writes it.
pub fn format(seconds: i64) -> String {
    format_with_micros(seconds, 0)
}

fn format_with_micros(seconds: i64, micros: u32) -> String {
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let rest = seconds.rem_euclid(86_400);
    let stamp = format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    );
    if micros == 0 {
        stamp
    } else {
        format!("{stamp}.{micros:06}")
    }
}

/// The wall clock, for `created_at` and `updated_at`: whole seconds, and the
/// time as it is written.
pub fn now() -> (i64, String) {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = elapsed.as_secs() as i64;
    (
        seconds,
        format_with_micros(seconds, elapsed.subsec_micros()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_story_time() {
        let at = parse("2026-09-06 19:00:00").unwrap();
        assert_eq!(at, 1_788_721_200);
        assert_eq!(format(at), "2026-09-06 19:00:00");
    }

    #[test]
    fn drops_the_fraction() {
        assert_eq!(
            parse("2026-09-27 13:51:11.674333"),
            parse("2026-09-27 13:51:11")
        );
        assert_eq!(format(-1), "1969-12-31 23:59:59");
    }
}
