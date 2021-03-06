use regex::Regex;
use chrono::*;

pub fn parse_datetime<Tz: TimeZone>(s: &str, now: DateTime<Tz>) -> Option<DateTime<Tz>> {
    lazy_static! {
        static ref DATE_WORDS_RE: Regex = Regex::new(r"today|yesterday|now").unwrap();
        static ref DATE_ABSOLUTE_RE: Regex = Regex::new(r"(?P<day>\d{1,2})/(?P<month>\d{1,2})(/(?P<year>\d{4}|\d{2}))?").unwrap();
        static ref DATE_RELATIVE_RE: Regex = Regex::new(r"(?P<n>\d+) (days?|ds?)(?P<ago>( ago)?)").unwrap();
        static ref TIME_ABSOLUTE_RE: Regex = Regex::new(r"(?P<hr>\d{2}):(?P<mins>\d{2})").unwrap();
        static ref TIME_RELATIVE_RE: Regex = Regex::new(r"(?x)
            (?P<n>\d+) \s (?P<dur>hrs?|hours?|hs?|minutes?|mins?|ms?) (?P<ago>(\s ago)?)").unwrap();
    }

    let now = if let Some(caps) = DATE_WORDS_RE.captures(s) {
        match caps.at(0) {
            Some("now") | Some("today") | None => now,
            Some("yesterday") => now - Duration::days(1),
            _ => unreachable!(),
        }
    } else if let Some(caps) = DATE_ABSOLUTE_RE.captures(s) {
        let day = caps.name("day").unwrap().parse().unwrap();
        let month = caps.name("month").unwrap().parse().unwrap();
        let current_year: i32 = Local::now().year() / 1000;
        match caps.name("year") {
            Some(s) if s.len() == 2 =>
                now.with_year(1000 * current_year + s.parse::<i32>().unwrap()).unwrap(),
            Some(s) if s.len() == 4 => now.with_year(s.parse().unwrap()).unwrap(),
            None => now,
            _ => unreachable!(),
        }.with_month(month).unwrap().with_day(day).unwrap()
    } else if let Some(caps) = DATE_RELATIVE_RE.captures(s) {
        let n = caps.name("n").unwrap().parse().unwrap();
        let duration = Duration::days(n);
        match caps.name("ago") {
            Some(s) if s.ends_with("ago") => now - duration,
            _ => now + duration,
        }
    } else {
        now
    };

    let now = if let Some(caps) = TIME_ABSOLUTE_RE.captures(s) {
        let hr = caps.name("hr").unwrap().parse().unwrap();
        let now = now.with_hour(hr).unwrap();
        let min = caps.name("mins").unwrap().parse().unwrap();
        now.with_minute(min).unwrap()
    } else if let Some(caps) = TIME_RELATIVE_RE.captures(s) {
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
