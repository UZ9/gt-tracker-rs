use chrono::prelude::*;
use clap::Parser;
use course::Course;

mod course;

/*
 * Main CLI usage:
 * python src/tracker.py [SEASON] CRN-1 CRN-2 ..
 */
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
            Season::Fall => 08,
            Season::Spring => 02,
            Season::Summer => 05,
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

pub fn run() {
    let args = Args::parse();
    env_logger::init();

    for crn in args.crns {
        let course = match Course::new(crn.to_string(), args.season) {
            Ok(course) => course,
            Err(e) => panic!("oopsy daisy {}", e),
        };
        println!("the course is {:?}\n", &course);
    }
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
