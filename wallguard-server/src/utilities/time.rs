#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(&'static str),
    InvalidNumber(std::num::ParseIntError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::InvalidNumber(e) => write!(f, "Invalid number: {}", e),
        }
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        ParseError::InvalidNumber(e)
    }
}

pub fn datetime_to_timestamp(date: &str, time: &str) -> Result<i64, ParseError> {
    let date_parts: Vec<&str> = date.split('/').collect();
    if date_parts.len() != 3 {
        return Err(ParseError::InvalidFormat("date must be YYYY/MM/DD"));
    }

    let time_parts: Vec<&str> = time.split(':').collect();
    if time_parts.len() != 2 {
        return Err(ParseError::InvalidFormat("time must be HH:MM"));
    }

    let year = date_parts[0].parse::<i64>()?;
    let month = date_parts[1].parse::<i64>()?;
    let day = date_parts[2].parse::<i64>()?;
    let hour = time_parts[0].parse::<i64>()?;
    let minute = time_parts[1].parse::<i64>()?;

    if !(1..=12).contains(&month) {
        return Err(ParseError::InvalidFormat("month must be 1-12"));
    }
    if !(1..=31).contains(&day) {
        return Err(ParseError::InvalidFormat("day must be 1-31"));
    }
    if !(0..=23).contains(&hour) {
        return Err(ParseError::InvalidFormat("hour must be 0-23"));
    }
    if !(0..=59).contains(&minute) {
        return Err(ParseError::InvalidFormat("minute must be 0-59"));
    }

    let days = days_from_epoch(year, month, day);
    Ok(days * 86400 + hour * 3600 + minute * 60)
}

fn days_from_epoch(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn timestamp_to_datetime(timestamp: i64) -> (String, String) {
    let seconds_in_day = 86400i64;

    let days = timestamp.div_euclid(seconds_in_day);
    let time_of_day = timestamp.rem_euclid(seconds_in_day);

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;

    let (year, month, day) = epoch_to_date(days);

    let date = format!("{:04}/{:02}/{:02}", year, month, day);
    let time = format!("{:02}:{:02}", hour, minute);

    (date, time)
}

fn epoch_to_date(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d)
}
