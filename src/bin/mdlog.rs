extern crate chrono;
extern crate structopt;

use chrono::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Input {
    /// The year to start from  
    /// defaults to the current local time year
    #[structopt(long = "year")]
    year: Option<i32>,
    /// First week to generate a MDLog template for  
    /// Weeks are numbered starting from 1; Thus any value ∊ [1,52] is accepted
    #[structopt(name = "weeknum")]
    week: u32,
    /// The number of weeks to generate  
    #[structopt(name = "n_weeks", default_value = "1")]
    n_weeks: u32,
}

/// The date formatting to use
const DFMT: &str = "%d.%m.%Y";

/// always print to stderr because we do use stdout for the generated templates
fn main() {
    let input = Input::from_args();
    assert!(
        0 < input.week && input.week <= 52,
        "Input week must be within [1,52]"
    );
    assert!(
        input.n_weeks >= 1u32,
        "Why would you use me to generate nothing?
         Come back when you want generate more than 0 weeks"
    );

    let year = input.year.unwrap_or_else(|| {
        let yr = Local::now().year();
        eprintln!("No year provided, defaulting to {}", yr);
        yr
    });

    eprintln!(
        "Generating templates for {} weeks starting with week {} of year {}",
        input.n_weeks, input.week, year
    );

    // correct for 1 week so this prints 1 week instead of 2 when given 1 as an input

    let mut day = NaiveDate::from_isoywd(year, input.week, Weekday::Mon);
    let last_day = {
        if input.week + input.n_weeks - 1 > 52 {
            // we get into the next year
            let endyear = year + (input.week + input.n_weeks) as i32 / 52;
            // since weeks start at 1 we need to compensate for that
            let last_week = (input.week + input.n_weeks - 1) % 52 + 1;
            println!("End: {}, m: {}", endyear, last_week);
            NaiveDate::from_isoywd(endyear, last_week, Weekday::Sun)
        } else {
            // we stay in the same year
            let endyear = year;
            let last_week = input.week + input.n_weeks - 1;
            NaiveDate::from_isoywd(endyear, last_week, Weekday::Sun)
        }
    };

    while day <= last_day {
        // generate a heading every time we begin a week
        if day.weekday() == Weekday::Mon {
            let end_of_week =
                NaiveDate::from_isoywd(day.year(), day.iso_week().week(), Weekday::Sun);
            println!(
                "# Week {}, {} - {}\n",
                day.iso_week().week(),
                day.format(DFMT),
                end_of_week.format(DFMT)
            );
        }
        println!("## {:?}, {}", day.weekday(), day.format(DFMT));
        println!("- TODO:  \n");

        // next day
        day = day.succ();
    }
    eprintln!("Done");
}
