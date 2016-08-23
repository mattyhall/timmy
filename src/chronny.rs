use regex::Regex;
use chrono::*;

pub fn parse_datetime<Tz: TimeZone>(s: &str, now: DateTime<Tz>) -> Option<DateTime<Tz>> {
    let date_re = Regex::new(r"(?x)
        today|yesterday|now|
        ((?P<n>\d+) \s days? (?P<ago>(\s ago)?))").unwrap();
    let time_absolute_re = Regex::new(r"(?P<hr>\d{2})(:(?P<secs>\d{2}))?").unwrap();
    let time_relative_re = Regex::new(r"(?P<in>(in)?) (?P<n>\d+) (?P<delim>hrs|hours|h|minutes|mins|m) (?P<ago>(ago))?").unwrap();

    let date_caps = date_re.captures(s).unwrap_or(date_re.captures("now").unwrap());

    let now = match date_caps.at(0) {
        Some("now") | Some("today") | None => now,
        Some("yesterday") => now - Duration::days(1),
        _ => {
            let n: i64 = date_caps.name("n").unwrap().parse().unwrap();
            let duration = Duration::days(n);
            match date_caps.name("ago") {
                Some(" ago") => now - duration,
                _ => now + duration,
            }
        }
    };

    let now = if let Some(caps) = time_absolute_re.captures(s) {
        println!("{:?} {:?}", caps.name("hr"), caps.name("secs"));
        now
    } else {
        now
    };
    Some(now)
}
