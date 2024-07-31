use scraper::{Html, Selector};

use crate::Season;

const BASE_URL: &str = "https://oscar.gatech.edu/bprod/bwckschd.p_disp_detail_sched";

// Queries for oscar page
const COURSE_NAME_SELECTOR: &str = "th.ddlabel";
const PREREQUISITE_SELECTOR: &str = "td.dddefault";

fn parse_element<'a>(
    document: &'a Html,
    selector_str: &str,
) -> Result<scraper::ElementRef<'a>, CourseError> {
    Selector::parse(selector_str)
        .map_err(|_| CourseError::ParseError(format!("Unable to parse selector: {}", selector_str)))
        .and_then(|selector| {
            document
                .select(&selector)
                .next()
                .ok_or(CourseError::ParseError(format!(
                    "Unable to parse element for selector: {}",
                    selector_str
                )))
        })
}

#[derive(Debug, Clone)]
pub struct Course {
    crn: String,
    season: Season,
    name: String,
    class_enrollment: Enrollment,
    waitlist_enrollment: Enrollment,
}

impl Course {
    pub fn crn(&self) -> &String {
        &self.crn
    }

    pub fn season(&self) -> &Season {
        &self.season
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn class_enrollment(&self) -> &Enrollment {
        &self.class_enrollment
    }

    pub fn waitlist_enrollment(&self) -> &Enrollment {
        &self.waitlist_enrollment
    }

    // TODO: Avoid using clone
    pub fn ref_array(&self) -> [String; 5] {
        [
            self.name.clone(),
            self.crn.clone(),
            self.class_enrollment.capacity.to_string(),
            self.class_enrollment.actual.to_string(),
            self.class_enrollment.remaining.to_string(),
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Enrollment {
    capacity: u32,
    actual: u32,
    remaining: u32,
}

impl Enrollment {
    pub fn remaining(&self) -> u32 {
        self.remaining
    }
}

#[derive(Debug)]
enum CourseError {
    ParseError(String),
}

impl std::fmt::Display for CourseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CourseError::ParseError(e) => write!(f, "Unable to parse: {}", e),
        }
    }
}

impl std::error::Error for CourseError {}

impl Course {
    pub fn new(crn: String, season: Season) -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("Creating new CRN: [{}, {}]", crn, season.get_term());

        let document = fetch_document(season, &crn)?;

        let course_name = parse_element(&document, COURSE_NAME_SELECTOR)?
            .text()
            .collect::<String>();

        let (class_enrollment, waitlist_enrollment) = parse_enrollment(document)?;

        Ok(Self {
            crn,
            season,
            name: course_name,
            class_enrollment,
            waitlist_enrollment,
        })
    }
}

fn get_row_content(row: scraper::ElementRef) -> Vec<String> {
    let td_selector = Selector::parse("td").unwrap();

    row.select(&td_selector)
        .map(|td| td.text().collect::<String>())
        .collect::<Vec<String>>()
}

fn parse_enrollment_data(data: Vec<String>) -> Result<Enrollment, CourseError> {
    let parsed_numbers: Result<Vec<u32>, _> = data.iter().map(|text| text.parse::<u32>()).collect();
    let parsed_numbers = parsed_numbers
        .map_err(|_| CourseError::ParseError("Unable to parse enrollment data".to_string()))?;
    if parsed_numbers.len() != 3 {
        return Err(CourseError::ParseError(
            "Expected 3 items within table row while parsing enrollment data".to_string(),
        ));
    }

    Ok(Enrollment {
        capacity: parsed_numbers[0],
        actual: parsed_numbers[1],
        remaining: parsed_numbers[2],
    })
}

fn parse_enrollment(
    document: Html,
) -> Result<(Enrollment, Enrollment), Box<dyn std::error::Error>> {
    let data_table = parse_element(&document, PREREQUISITE_SELECTOR)?;
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let mut rows = data_table.select(&tr_selector);
    rows.next();
    let seat_tr = rows.next().ok_or(CourseError::ParseError(
        "Unable to parse seat enrollment tr".to_string(),
    ))?;
    let waitlist_seat_tr = rows.next().ok_or(CourseError::ParseError(
        "Unable to parse waitlist seat enrollment tr".to_string(),
    ))?;

    let class_enrollment = parse_enrollment_data(get_row_content(seat_tr))?;
    let waitlist_enrollment = parse_enrollment_data(get_row_content(waitlist_seat_tr))?;
    Ok((class_enrollment, waitlist_enrollment))
}

fn fetch_document(season: Season, crn: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let body: String = ureq::get(BASE_URL)
        .query("term_in", &season.get_term())
        .query("crn_in", crn)
        .call()?
        .into_string()?;

    Ok(Html::parse_document(&body))
}
