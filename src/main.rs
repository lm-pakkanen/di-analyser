mod fns;
mod util;

use crate::fns::csv::read_csv;
use fns::{
    diff::calculate_diffs,
    fs::{write_correlation_data, write_diffs_data, write_impact_data},
    impact::get_impact_averages,
    rank::calculate_rankings,
    util::assign_ids,
};
use std::process;
use util::types::{QuestionCorrelationData, QuestionDataDiffs, QuestionImpactData};

fn main() {
    let filename = "./data/feedbacks.csv";

    match read_csv(filename) {
        Ok(mut feedbacks) => {
            assign_ids(&mut feedbacks);

            let rankings: Vec<QuestionCorrelationData> = calculate_rankings(&feedbacks);
            let impact_averages: Vec<QuestionImpactData> = get_impact_averages(&feedbacks);
            let diffs: Vec<QuestionDataDiffs> = calculate_diffs(&rankings, &impact_averages);

            write_correlation_data(&rankings);
            write_impact_data(&impact_averages);
            write_diffs_data(&diffs);
        }
        Err(err) => {
            println!("Failed to read CSV: {}", err);
            process::exit(1);
        }
    }
}
