extern crate chrono;
extern crate timmy;

use chrono::*;
use timmy::chronny::*;

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2016-08-23T16:30:00+01:00").unwrap()
}

#[test]
fn test_now() {
    assert_eq!(parse_datetime("now", now()), Some(now()));
    assert_eq!(parse_datetime("today", now()), Some(now()));
}

#[test]
fn test_yesterday() {
    let yesterday = now() - Duration::days(1);
    assert_eq!(parse_datetime("yesterday", now()), Some(yesterday));
}

#[test]
fn test_relative_date() {
    let yesterday = now() - Duration::days(1);
    let three_days_ago = now() - Duration::days(3);
    let in_four_days = now() + Duration::days(4);
    assert_eq!(parse_datetime("1 day ago", now()), Some(yesterday));
    assert_eq!(parse_datetime("3 days ago", now()), Some(three_days_ago));
    assert_eq!(parse_datetime("in 4 days", now()), Some(in_four_days));
}

#[test]
fn test_absolute_time() {
    let in_one_hour = now() + Duration::hours(1);
    let two_hours_ago = now() - Duration::hours(2);
}
