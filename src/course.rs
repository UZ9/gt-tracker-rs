use crate::{oscar_scraper::get_course_name, Season};

pub struct Course {
    pub crn: String,
    pub season: Season,
    pub name: String,
}

impl Course {
    pub fn new(crn: String, season: Season) -> Result<Self, ureq::Error> {
        log::info!("Creating new CRN: [{}, {}]", crn, season.get_term());
        let name = match get_course_name(&season, &crn) {
            Ok(course_name) => course_name,
            Err(e) => return Err(e),
        };

        let new_self = Self { crn, season, name };

        Ok(new_self)
    }
}
