use scraper::{Html, Selector};

use crate::Season;

const BASE_URL: &str = "https://oscar.gatech.edu/bprod/bwckschd.p_disp_detail_sched";

// Queries for oscar page
const COURSE_NAME_SELECTOR: &str = "th.ddlabel";

fn parse_element<'a>(document: &'a Html, selector_str: &str) -> Option<scraper::ElementRef<'a>> {
    Selector::parse(selector_str)
        .ok()
        .and_then(|selector| document.select(&selector).next())
}

pub fn get_course_info(season: &Season, crn: &String) -> Result<String, ureq::Error> {
    let body: String = ureq::get(BASE_URL)
        .query("term_in", &season.get_term())
        .query("crn_in", crn)
        .call()?
        .into_string()?;

    let document = Html::parse_document(&body);

    let course_name = parse_element(&document, "th.ddlabel")
        .unwrap()
        .text()
        .collect::<String>();

    Ok(course_name)
}
