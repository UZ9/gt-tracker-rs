use kdam::tqdm;
use std::fmt::Display;

use chrono::prelude::*;
use clap::Parser;
use course::Course;
// Length of the CRN identifier for a class, e.g. 239485
const CRN_LENGTH: usize = 6;

pub mod course;

#[derive(Parser, Debug)]
struct Args {
    /// The semester to watch classes for
    #[arg(short, long, required = true)]
    season: Season,

    /// A list of class CRNs to filter through
    #[arg(short, long, required=true, num_args=1..)]
    crns: Vec<String>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Season {
    Fall,
    Spring,
    Summer,
}

impl Season {
    pub fn get_id(&self) -> i32 {
        match self {
            Season::Fall => 8,
            Season::Spring => 2,
            Season::Summer => 5,
        }
    }

    pub fn get_year(&self) -> i32 {
        let now = chrono::Utc::now();
        let month = now.month();
        let mut year = now.year();

        // Because the spring semester is potentially the next year,
        // we add an additional check for this scenario.
        if *self == Season::Spring && month > 4 {
            year += 1;
        }

        year
    }

    pub fn get_term(&self) -> String {
        let year = self.get_year();
        let id = self.get_id();

        format!("{}0{}", year, id)
    }
}

impl Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Season::Fall => write!(f, "Fall"),
            Season::Spring => write!(f, "Spring"),
            Season::Summer => write!(f, "Summer"),
        }
    }
}

pub fn get_input_courses() -> Vec<course::Course> {
    let args = Args::parse();
    env_logger::init();

    let crns: Vec<Course> = tqdm!(args.crns.into_iter())
        .map(|crn| Course::new(crn.to_string(), args.season).unwrap())
        .collect();

    crns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_is_correct() {
        let now = chrono::Utc::now();
        let month = now.month();

        let year = if month > 4 {
            now.year() + 1
        } else {
            now.year()
        };

        assert_eq!(Season::Spring.get_year(), year);
    }

    #[test]
    fn term_is_correct() {
        let id = Season::Spring.get_id();
        let year = Season::Spring.get_year();

        let term = format!("{}0{}", year, id);

        assert_eq!(Season::Spring.get_term(), term);
    }
}
