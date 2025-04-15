use crate::util::types::Feedback;
use std::{error::Error, fs, path::Path};

pub fn read_csv<P: AsRef<Path>>(filename: P) -> Result<Vec<Feedback>, Box<dyn Error>> {
    let content = match get_csv_content(filename) {
        Ok(content) => content,
        Err(err) => {
            panic!("Failed to read CSV: {}", err);
        }
    };

    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let mut feedbacks: Vec<Feedback> = Vec::new();

    for result in reader.deserialize() {
        let record: Feedback = result?;
        feedbacks.push(record);
    }

    Ok(feedbacks)
}

fn get_csv_content<P: AsRef<Path>>(filename: P) -> Result<String, Box<dyn Error>> {
    let content: String = fs::read_to_string(filename)?;
    let reduced_content = remove_boilerplate_content(content);
    return Ok(reduced_content);
}

fn remove_boilerplate_content(content: String) -> String {
    let replaced = content.replace("In general, how impactful on quality are the following development practices in your opinion?\n1 means not impactful at all and 5 means very impactful.  ", "");
    return replaced;
}
