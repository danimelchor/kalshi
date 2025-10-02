use chrono::DateTime;
use chrono_tz::Tz;

pub fn check_dates_match(d1: &DateTime<Tz>, d2: &DateTime<Tz>) -> bool {
    d1.with_timezone(&d2.timezone()).date_naive() == d2.date_naive()
}
