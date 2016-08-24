use regex::Regex;
use chrono::*;

pub fn parse_datetime<Tz: TimeZone>(s: &str, now: DateTime<Tz>) -> Option<DateTime<Tz>> {
    let date_re = Regex::new(r"(?x)
        today|yesterday|now|
        ((?P<n>\d+) \s (days?|ds?) (?P<ago>(\s ago)?))").unwrap();
    let time_absolute_re = Regex::new(r"(?P<hr>\d{2}):(?P<mins>\d{2})").unwrap();
    let time_relative_re = Regex::new(r"(?x)
        (?P<n>\d+) \s (?P<dur>hrs?|hours?|hs?|minutes?|mins?|ms?) (?P<ago>(\s ago)?)").unwrap();

    let date_caps = date_re.captures(s).unwrap_or(date_re.captures("now").unwrap());

    let now = match date_caps.at(0) {
        Some("now") | Some("today") | None => now,
        Some("yesterday") => now - Duration::days(1),
        _ => {
            let n = date_caps.name("n").unwrap().parse().unwrap();
            let duration = Duration::days(n);
            match date_caps.name("ago") {
                Some(s) if s.ends_with("ago") => now - duration,
                _ => now + duration,
            }
        }
    };

    let now = if let Some(caps) = time_absolute_re.captures(s) {
        let hr = caps.name("hr").unwrap().parse().unwrap();
        let now = now.with_hour(hr).unwrap();
        let min = caps.name("mins").unwrap().parse().unwrap();
        now.with_minute(min).unwrap()
    } else if let Some(caps) = time_relative_re.captures(s) {
        let n = caps.name("n").unwrap().parse().unwrap();
        let dur = caps.name("dur").unwrap();
        let duration = if dur.starts_with("h") {
            Duration::hours(n)
        } else {
            Duration::minutes(n)
        };
        match caps.name("ago") {
            Some(s) if s.ends_with("ago") => now - duration,
            _ => now + duration
        }
    } else {
        now
    };
    Some(now)
}
