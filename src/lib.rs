extern crate chrono;
extern crate serde;
extern crate serde_yaml;

pub mod parser;

pub mod types {
    use chrono::naive::{NaiveDate, NaiveTime};
    use chrono::Datelike;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Hash, Clone, Deserialize, Eq, PartialEq, Debug)]
    pub struct Person {
        pub name: String,
        pub birthday: Birthday,
        pub presents: Option<Vec<String>>,
    }

    #[derive(Serialize, Hash, Clone, Deserialize, Eq, PartialEq, Debug)]
    pub enum Birthday {
        /// Full Date
        KnownYear(NaiveDate),
        /// Year, Month
        UnknownYear(u32, u32),
    }

    #[derive(Serialize, Hash, Clone, Deserialize, Eq, PartialEq, Debug)]
    pub struct Task {
        pub msg: String,
        pub subtasks: Vec<Subtask>,
        pub notes: Vec<String>,
        pub date: NaiveDate,
        pub is_done: bool,
    }

    #[derive(Serialize, Hash, Clone, Deserialize, Eq, PartialEq, Debug)]
    pub struct Subtask {
        pub msg: String,
        pub is_done: bool,
    }

    #[derive(Serialize, Hash, Clone, Deserialize, Eq, PartialEq, Debug)]
    pub struct Event {
        pub msg: String,
        pub notes: Vec<String>,
        pub date: NaiveDate,
        pub time: Option<NaiveTime>,
    }

    impl Birthday {
        pub fn day(&self) -> u32 {
            match self {
                Self::KnownYear(d) => d.day(),
                Self::UnknownYear(_, d) => *d,
            }
        }

        pub fn month(&self) -> u32 {
            match self {
                Self::KnownYear(d) => d.month(),
                Self::UnknownYear(m, _) => *m,
            }
        }
    }

}
