use scraper::{Html, Selector};

use crate::Season;

pub fn get_course_name(season: &Season, crn: &String) -> Result<String, ureq::Error> {
    let body: String = ureq::get("https://oscar.gatech.edu/bprod/bwckschd.p_disp_detail_sched")
        .query("term_in", &season.get_term())
        .query("crn_in", crn)
        .call()?
        .into_string()?;

    let document = Html::parse_document(&body);

    let binding = r#"th.ddlabel"#;
    let header_selector = Selector::parse(binding).unwrap();

    let header_title = document
        .select(&header_selector)
        .next()
        .unwrap()
        .text()
        .collect::<String>();

    Ok(header_title.to_string())
}
