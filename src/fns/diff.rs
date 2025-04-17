use crate::util::{
    types::{QuestionCorrelationData, QuestionDataDiffs, QuestionImpactData},
    vars,
};

pub fn calculate_diffs(
    correlations: &Vec<QuestionCorrelationData>,
    impacts: &Vec<QuestionImpactData>,
) -> Vec<QuestionDataDiffs> {
    let mut diffs: Vec<QuestionDataDiffs> = Vec::new();

    for ranking in correlations.iter() {
        let impact_field_name: &str = get_impact_field_name(&ranking.question);

        let impact = impacts
            .iter()
            .find(|n| n.question == impact_field_name)
            .unwrap();

        let comparable_rho = ranking.rho * vars::COMPARABLE_RHO_MULTIPLIER;

        let diff = QuestionDataDiffs {
            question_correlation: ranking.question.to_owned(),
            rho: ranking.rho,
            p_value: ranking.p_value,
            correlation_answer_count: ranking.answer_count,
            question_impact: impact.question.to_owned(),
            impact_average: impact.impact_average,
            impact_answer_count: impact.answer_count,
            diff: comparable_rho - impact.impact_average as f64,
        };
        diffs.push(diff);
    }

    diffs.sort_by(|a, b| b.rho.partial_cmp(&a.rho).unwrap());
    diffs
}

fn get_impact_field_name(correlation_field_name: &str) -> &str {
    return match correlation_field_name {
        "were_semi_automated_processes_run" => "impact_semi_automated_ci_cd",
        "was_code_style_enforced" => "impact_code_style",
        "were_higher_level_design_issues_considered" => "impact_higher_level_design",
        "were_unit_tests_written_with_boc" => "impact_unit_tests_boc",
        "were_todo_comments_avoided" => "impact_avoiding_todo_comments",
        "were_technical_design_documents_written" => "impact_writing_technical_design_documents",
        "were_tests_written_like_production_code" => "impact_tests_like_production",
        "were_nulls_avoided" => "impact_nulls",
        "was_non_english_used" => "impact_using_english",
        "were_requirements_documents_written" => "impact_writing_requirements_documents",
        "were_automated_tests_run" => "impact_automated_tests",
        "was_reviewer_required" => "impact_requiring_reviewer",
        "were_specification_documents_written" => "impact_writing_specification_documents",
        "were_formatter_and_linter_tools_established" => "impact_formatter_linter_tools",
        "were_static_analysis_tools_established" => "impact_static_analysis_tools",
        "was_commit_message_format_established" => "impact_commit_message_format",
        "were_regex_patterns_commented" => "impact_regex_comments",
        "were_magic_numbers_replaced" => "impact_magic_numbers",
        "was_merging_strategy_established" => "impact_merging_strategy",
        "were_mutability_and_side_effects_avoided" => "impact_avoiding_mutability",
        "were_draft_design_documents_written" => "impact_writing_draft_design_documents",
        "was_initial_project_plan_written" => "impact_writing_initial_project_plan",
        "were_mutable_names_encoded" => "impact_encoding_mutable_names",
        "were_semi_automated_tools_configured" => "impact_centralising_tools",
        "was_branch_naming_strategy_established" => "impact_branch_naming_strategy",
        "was_code_refactored" => "impact_code_refactoring",
        "was_sbom_document_written" => "impact_writing_sbom_document",
        "were_critical_code_commented" => "impact_code_comments",
        "was_branching_strategy_established" => "impact_branching_strategy",
        "were_project_documents_reviewed" => "impact_requiring_document_reviewers",
        "were_posix_timestamps_used" => "impact_posix_timestamps",
        "was_ai_generated_code_reviewed" => "impact_reviewing_ai_code",
        _ => panic!("Unknown field name"),
    };
}
