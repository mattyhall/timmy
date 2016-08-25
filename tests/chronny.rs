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
fn test_absolute_date() {
    let yesterday = now() - Duration::days(1);
    let first = DateTime::parse_from_rfc3339("2016-08-01T16:30:00+01:00").unwrap();
    assert_eq!(parse_datetime("yesterday", now()), Some(yesterday));
    assert_eq!(parse_datetime("01/08/16", now()), Some(first));
    assert_eq!(parse_datetime("01/08/2016", now()), Some(first));
    assert_eq!(parse_datetime("1/8/2016", now()), Some(first));
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
    assert_eq!(parse_datetime("17:30", now()), Some(in_one_hour));
    assert_eq!(parse_datetime("14:30", now()), Some(two_hours_ago));
}

#[test]
fn test_relative_time() {
    let in_one_hour = now() + Duration::hours(1);
    let in_thirty_mins = now() + Duration::minutes(30);
    let three_hours_ago = now() - Duration::hours(3);
    assert_eq!(parse_datetime("in 1 hr", now()), Some(in_one_hour));
    assert_eq!(parse_datetime("in 1 hours", now()), Some(in_one_hour));
    assert_eq!(parse_datetime("in 30 mins", now()), Some(in_thirty_mins));
    assert_eq!(parse_datetime("in 30 minutes", now()), Some(in_thirty_mins));
    assert_eq!(parse_datetime("3 hrs ago", now()), Some(three_hours_ago));
}

#[test]
fn test_kitchen_sink() {
    let yesterday_two = now() - Duration::days(1) - Duration::hours(2) - Duration::minutes(30);
    assert_eq!(parse_datetime("yesterday 14:00", now()), Some(yesterday_two));
}
