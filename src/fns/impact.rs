use crate::util::{
    types::{Feedback, QuestionImpactData},
    vars,
};
use std::collections::HashMap;

macro_rules! handle_feedback {
  ( $impact_map:expr, $feedbacks:expr, $($field:ident),* ) => {
        for feedback in $feedbacks {
            $(
                if let Ok(v) = feedback.$field.parse::<f32>() {
                    $impact_map.entry(stringify!($field).to_owned()).or_insert(vec![]).push(v);
                }
            )*
        }
    }
}

pub fn get_impact_averages(feedbacks: &Vec<Feedback>) -> Vec<QuestionImpactData> {
    let mut impact_map: HashMap<String, Vec<f32>> = HashMap::new();

    handle_feedback!(
        impact_map,
        feedbacks,
        impact_writing_requirements_documents,
        impact_writing_initial_project_plan,
        impact_writing_specification_documents,
        impact_writing_technical_design_documents,
        impact_writing_sbom_document,
        impact_writing_draft_design_documents,
        impact_requiring_document_reviewers,
        impact_avoiding_mutability,
        impact_encoding_mutable_names,
        impact_nulls,
        impact_using_english,
        impact_code_style,
        impact_code_refactoring,
        impact_posix_timestamps,
        impact_magic_numbers,
        impact_code_comments,
        impact_regex_comments,
        impact_avoiding_todo_comments,
        impact_reviewing_ai_code,
        impact_branching_strategy,
        impact_branch_naming_strategy,
        impact_commit_message_format,
        impact_merging_strategy,
        impact_higher_level_design,
        impact_requiring_reviewer,
        impact_formatter_linter_tools,
        impact_static_analysis_tools,
        impact_centralising_tools,
        impact_semi_automated_ci_cd,
        impact_automated_tests,
        impact_tests_like_production,
        impact_unit_tests_boc
    );

    let mut impact_data: Vec<QuestionImpactData> = impact_map
        .iter()
        .map(|(key, value)| {
            let impact_sum: f32 = value.iter().sum::<f32>();
            let answer_count: usize = value.len();
            let impact_average: f32 = impact_sum / answer_count as f32;

            let impact_average_rounded: f32 =
                (impact_average * vars::ROUND_DECIMAL_PLACES_MULTIPLIER as f32).round()
                    / vars::ROUND_DECIMAL_PLACES_MULTIPLIER as f32;

            QuestionImpactData {
                question: key.to_owned(),
                impact_average: impact_average_rounded,
                answer_count,
            }
        })
        .collect();
    impact_data.sort_by(|a, b| b.impact_average.partial_cmp(&a.impact_average).unwrap());
    impact_data
}

pub fn get_impact_average(data: &Vec<QuestionImpactData>) -> f64 {
    let impacts_sum: f64 = data.iter().map(|x| x.impact_average as f64).sum();
    let question_count: usize = data.len();
    let impact_average: f64 = impacts_sum / question_count as f64;

    let impact_average_rounded: f64 =
        (impact_average * vars::ROUND_DECIMAL_PLACES_MULTIPLIER as f64).round()
            / vars::ROUND_DECIMAL_PLACES_MULTIPLIER as f64;

    impact_average_rounded
}
