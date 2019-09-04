extern crate chrono;
extern crate mdlog;
extern crate structopt;

use chrono::{Datelike, Local, NaiveDate, Weekday};
use structopt::StructOpt;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use mdlog::parser;
use mdlog::types::Person;

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
    #[structopt(flatten)]
    bd_config: BD,
}

#[derive(Debug, StructOpt)]
struct BD {
    /// The yaml file to include birthdays from.
    /// The file should be in the form of a dict of name, date. Example:
    /// ```
    /// Alex: 19.01.2001
    /// Bob: 20.12.?
    /// ```
    #[structopt(
        long = "birthday-file",
        default_value = "birthdays.yml",
        help = "The file to source birthdays from."
    )]
    bd_file: PathBuf,
    /// Whether to includes birthdates of people mentioned in the birthday file when generating templates.
    #[structopt(short = "bd", long = "generate-birthdays")]
    include_birthdays: bool,
    /// Whether to randomly include a todo to call someone from the birthday file when generating templates.
    /// Makes it a little easier to stay in touch
    #[structopt(short = "ca", long = "generate-calls")]
    gen_calls: bool,
}

/// The date formatting to use
const DATE_FMT: &str = "%d.%m.%Y";

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

    let today = Local::today().naive_local();

    let year = input.year.unwrap_or_else(|| {
        eprintln!("No year provided, defaulting to {}", today.year());
        today.year()
    });

    eprintln!(
        "Generating templates for {} weeks starting with week {} of year {}",
        input.n_weeks, input.week, year
    );

    let bds: HashMap<(u32, u32), Vec<Person>> =
        if input.bd_config.include_birthdays || input.bd_config.gen_calls {
            read_and_prep_birthday_file(&input.bd_config.bd_file)
        } else {
            HashMap::new()
        };

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
                day.format(DATE_FMT),
                end_of_week.format(DATE_FMT)
            );
        }

        println!("## {:?}, {}", day.weekday(), day.format(DATE_FMT));
        if input.bd_config.include_birthdays {
            if let Some(people) = bds.get(&(day.month(), day.day())) {
                people.iter().for_each(|p| {
                    println!(
                        "- TODO: Congratulate {} (Age {})",
                        p.name,
                        // FIXME: this may go wrong in some funky situations if there is a different number of weeks per year
                        (today - p.birthdate).num_weeks() / 52
                    )
                })
            }
        }
        if input.bd_config.gen_calls {
            // TOOD: pull in rand
            // TODO: create a vec of people
            // TODO: pick a bool with a certain probability
            // TODO: if the bool is true, pick a random person rome the list
            // TODO: println!("- TODO: Call {}", person)
            unimplemented!()
        }
        // insert an empty line
        #[allow(clippy::println_empty_string)]
        println!();

        // next day
        day = day.succ();
    }
    eprintln!("Done");
}

fn read_and_prep_birthday_file(file: &PathBuf) -> HashMap<(u32, u32), Vec<Person>> {
    let people = match parser::parse_birthdays_yaml(file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse birthday file with {}", e);
            process::exit(e.raw_os_error().unwrap_or(-1));
        }
    };

    let mut m = HashMap::new();
    people
        .into_iter()
        .map(|p| ((p.birthdate.month(), p.birthdate.day()), p))
        .for_each(|p| m.entry(p.0).or_insert_with(|| vec![]).push(p.1));
    m
}
