use scraper::{Html, Selector};

use crate::Season;

const BASE_URL: &str = "https://oscar.gatech.edu/bprod/bwckschd.p_disp_detail_sched";

// Queries for oscar page
const COURSE_NAME_SELECTOR: &str = "th.ddlabel";
const PREREQUISITE_SELETOR: &str = "td.dddefault";

fn parse_element<'a>(document: &'a Html, selector_str: &str) -> Option<scraper::ElementRef<'a>> {
    Selector::parse(selector_str)
        .ok()
        .and_then(|selector| document.select(&selector).next())
}

#[derive(Debug, Clone)]
pub struct Course {
    pub crn: String,
    pub season: Season,
    pub name: String,
    pub class_enrollment: Enrollment,
    pub waitlist_enrollment: Enrollment,
}

#[derive(Debug, Clone)]
struct Enrollment {
    capacity: u32,
    actual: u32,
    remaining: u32,
}

impl Course {
    pub fn new(crn: String, season: Season) -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("Creating new CRN: [{}, {}]", crn, season.get_term());
        let body: String = ureq::get(BASE_URL)
            .query("term_in", &season.get_term())
            .query("crn_in", &crn)
            .call()?
            .into_string()?;

        let document = Html::parse_document(&body);

        let course_name = parse_element(&document, COURSE_NAME_SELECTOR)
            .unwrap()
            .text()
            .collect::<String>();

        let data_table = parse_element(&document, PREREQUISITE_SELETOR).unwrap();

        let tr_selector = Selector::parse("tbody > tr").unwrap();
        let mut rows = data_table.select(&tr_selector);

        // Skip first row, as it's just the header names and not actual data
        rows.next();

        let seat_tr = rows.next().unwrap();
        let waitlist_seat_tr = rows.next().unwrap();

        let get_row_content = |row: scraper::ElementRef| {
            let td_selector = Selector::parse("td").unwrap();

            row.select(&td_selector)
                .map(|td| td.text().collect::<String>())
                .collect::<Vec<String>>()
        };

        let class_enrollment_data = get_row_content(seat_tr)
            .iter()
            .map(|text| text.parse::<u32>())
            .collect::<Result<Vec<u32>, _>>()?;

        if class_enrollment_data.len() != 3 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to find expected class enrollment fields",
            )));
        }

        let class_enrollment = Enrollment {
            capacity: class_enrollment_data[0],
            actual: class_enrollment_data[1],
            remaining: class_enrollment_data[2],
        };

        let waitlist_enrollment_data = get_row_content(waitlist_seat_tr)
            .iter()
            .map(|text| text.parse::<u32>())
            .collect::<Result<Vec<u32>, _>>()?;

        let waitlist_enrollment = Enrollment {
            capacity: waitlist_enrollment_data[0],
            actual: waitlist_enrollment_data[1],
            remaining: waitlist_enrollment_data[2],
        };

        Ok(Self {
            crn,
            season,
            name: course_name,
            class_enrollment,
            waitlist_enrollment,
        })
    }
}
