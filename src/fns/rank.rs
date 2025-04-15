use crate::{
    fns::util::get_answer_score,
    util::{
        types::{Feedback, FeedbackWithData, QuestionCorrelationData},
        vars,
    },
};
use statrs::distribution::{ContinuousCDF, StudentsT};

static IS_TRACE: bool = false;
static IS_DEBUG: bool = false;

pub fn calculate_rankings(feedbacks: &Vec<Feedback>) -> Vec<QuestionCorrelationData> {
    let mut result = get_feedbacks_with_ranks(feedbacks);
    result.sort_by(|a, b| b.rho.partial_cmp(&a.rho).unwrap());
    return result;
}

macro_rules! parse_score {
    ( $feedback:expr, $feedback_with_data:expr, $(($field:ident, $score_field:ident)),* ) => {
        $(
              let score = get_answer_score($feedback.$field.as_str());
              $feedback_with_data.$score_field = score;
        )*
    };
}

// NOTE filters out zeroes even with quality scores
macro_rules! parse_ranks {
    ( $feedbacks:expr, $(($( $score_field:ident ).+, $rank_field:ident)),* ) => {
      $(
          let mut applicable_feedbacks: Vec<_> = $feedbacks
              .iter_mut()
              .filter(|f| f.$($score_field).+ != vars::WEIGHTING_NOT_APPLICABLE)
              .collect();

          applicable_feedbacks.sort_by(|a, b| b.$($score_field).+.partial_cmp(&a.$($score_field).+).unwrap());

          let mut last_score: i8 = -10;
          let mut rank_sum: f64 = 0.0;
          let mut tie_count: usize = 0;

          for i in 0..applicable_feedbacks.len() {
              let current_rank = (i + 1) as f64;
              let feedback = &mut applicable_feedbacks[i];
              let score = feedback.$($score_field).+;

              if score == last_score {
                  rank_sum += current_rank;
                  tie_count += 1;
              } else {
                  if tie_count > 0 {
                      let average_rank = rank_sum / tie_count as f64;
                      for j in (i - tie_count)..i {
                          applicable_feedbacks[j].$rank_field = average_rank;
                      }
                  }
                  last_score = score;
                  rank_sum = current_rank;
                  tie_count = 1;
              }

              if i == applicable_feedbacks.len() - 1 {
                  let average_rank = rank_sum / tie_count as f64;
                  for j in (i + 1 - tie_count)..=i {
                      applicable_feedbacks[j].$rank_field = average_rank;
                  }
              }
          }

          if IS_DEBUG {
              println!(
                  "Scores sorted by {}: {:#?}",
                  stringify!($($score_field).+),
                  applicable_feedbacks.iter()
                      .map(|f| (
                          f.$($score_field).+,
                          f.$rank_field
                      ))
                      .collect::<Vec<_>>()
              );
          }
      )*

        if IS_TRACE {
            println!("Feedbacks: {:#?}", $feedbacks);
        }
    };
}

// NOTE filters out zeroes even with quality scores
macro_rules! parse_correlations {
    (
        $feedbacks_with_data:expr, $results:expr,
        $(( $question_name:ident, $( $score_field:ident ).+, $rank_field:ident, $spearman_field:ident, $p_value_field:ident )),*
    ) => {

        if IS_DEBUG {
            println!("Total answers: {}", $feedbacks_with_data.len());
        }

        $(
            let mut applicable_feedbacks: Vec<&mut FeedbackWithData> = $feedbacks_with_data
                .iter_mut()
                .filter(|f| f.$($score_field).+ != vars::WEIGHTING_NOT_APPLICABLE)
                .collect();

            let applicable_answer_count = applicable_feedbacks.len();

            if applicable_answer_count < 3 {
                panic!("Not enough applicable answers for correlation calculation: {}", applicable_answer_count);
            }

            // Parse ranks for the applicable feedbacks; ignore the rest in ranking
            parse_ranks!(
                applicable_feedbacks,
                (
                    feedback.project_quality_estimate,
                    project_quality_estimate_rank
                )
            );

            if IS_DEBUG {
              println!("### Question: {}", stringify!($question_name));
              println!("Applicable answer count: {}", applicable_answer_count);
            }

            let mean_rank_d = applicable_feedbacks
                .iter()
                .map(|f| f.$rank_field)
                .sum::<f64>() / applicable_answer_count as f64;

            let mean_rank_q = applicable_feedbacks
                .iter()
                .map(|f| f.project_quality_estimate_rank)
                .sum::<f64>() / applicable_answer_count as f64;

            if IS_DEBUG {
                println!("Mean rank Q: {:#.05} | Mean rank Quality: {:#.05}",
                  mean_rank_d,
                  mean_rank_q
                );
            }

            let mut rank_m_product_sum = 0f64;
            let mut rank_d_m_sq_sum = 0f64;
            let mut rank_q_m_sq_sum = 0f64;

            for feedback in applicable_feedbacks {
                let rank_d = feedback.$rank_field;
                let rank_q = feedback.project_quality_estimate_rank;
                let rank_d_m = rank_d - mean_rank_d;
                let rank_q_m = rank_q - mean_rank_q;
                let rank_m_product = rank_d_m * rank_q_m;
                let rank_d_m_sq = f64::powf(rank_d_m, 2f64);
                let rank_q_m_sq = f64::powf(rank_q_m, 2f64);

                rank_m_product_sum += rank_m_product;
                rank_d_m_sq_sum += rank_d_m_sq;
                rank_q_m_sq_sum += rank_q_m_sq;

                if IS_DEBUG {
                    println!(
                        "Rank Q: {:#.05} | Rank Quality: {:#.05} | Rank Q - Mean Rank Q: {:#.05} | Rank Quality - Mean Rank Quality: {:#.05} | Rank - Mean Rank Product: {:#.05}",
                        rank_d,
                        rank_q,
                        rank_d_m,
                        rank_q_m,
                        rank_m_product
                  );
                }
            }

          // See: https://en.wikipedia.org/wiki/Pearson_correlation_coefficient
          let rho = rank_m_product_sum / (f64::sqrt(rank_d_m_sq_sum) * f64::sqrt(rank_q_m_sq_sum));

          if rho < -1f64 || rho > 1f64 {
              panic!("Spearman's rho is out of bounds: {:#.05}", rho);
          }

          let t_test = rho * f64::sqrt((applicable_answer_count - 2) as f64 / (1f64 - f64::powf(rho, 2f64)));
          let p_value = get_p_value(t_test, applicable_answer_count);

          if p_value < -1f64 || p_value > 1f64 {
              panic!("p-value is out of bounds: {:#.05}", p_value);
          }

          if IS_DEBUG {
              println!("Spearman's rho: {:#.05} | p-value: {:#.05} | t-test: {:#.05}",
                rho,
                p_value,
                t_test,
              );
          }

            $results.push(
                QuestionCorrelationData {
                    question: stringify!($question_name).to_owned(),
                    rho,
                    p_value,
                    answer_count: applicable_answer_count,
                }
            );
        )*
    };
}

pub fn get_feedbacks_with_ranks(feedbacks: &Vec<Feedback>) -> Vec<QuestionCorrelationData> {
    let mut feedbacks_with_data: Vec<FeedbackWithData> = Vec::new();

    for feedback in feedbacks {
        let mut feedback_with_data: FeedbackWithData = FeedbackWithData::new(feedback.to_owned());

        parse_score!(
            &feedback,
            &mut feedback_with_data,
            (
                were_requirements_documents_written,
                were_requirements_documents_written_score
            ),
            (
                was_initial_project_plan_written,
                was_initial_project_plan_written_score
            ),
            (
                were_specification_documents_written,
                were_specification_documents_written_score
            ),
            (
                were_technical_design_documents_written,
                were_technical_design_documents_written_score
            ),
            (was_sbom_document_written, was_sbom_document_written_score),
            (
                were_draft_design_documents_written,
                were_draft_design_documents_written_score
            ),
            (
                were_project_documents_reviewed,
                were_project_documents_reviewed_score
            ),
            (
                were_mutability_and_side_effects_avoided,
                were_mutability_and_side_effects_avoided_score
            ),
            (were_mutable_names_encoded, were_mutable_names_encoded_score),
            (was_non_english_used, was_non_english_used_score),
            (was_code_style_enforced, was_code_style_enforced_score),
            (was_code_refactored, was_code_refactored_score),
            (were_posix_timestamps_used, were_posix_timestamps_used_score),
            (
                were_magic_numbers_replaced,
                were_magic_numbers_replaced_score
            ),
            (
                were_critical_code_commented,
                were_critical_code_commented_score
            ),
            (
                were_regex_patterns_commented,
                were_regex_patterns_commented_score
            ),
            (were_todo_comments_avoided, were_todo_comments_avoided_score),
            (
                was_ai_generated_code_reviewed,
                was_ai_generated_code_reviewed_score
            ),
            (
                was_branching_strategy_established,
                was_branching_strategy_established_score
            ),
            (
                was_branch_naming_strategy_established,
                was_branch_naming_strategy_established_score
            ),
            (
                was_commit_message_format_established,
                was_commit_message_format_established_score
            ),
            (
                was_merging_strategy_established,
                was_merging_strategy_established_score
            ),
            (
                were_higher_level_design_issues_considered,
                were_higher_level_design_issues_considered_score
            ),
            (was_reviewer_required, was_reviewer_required_score),
            (
                were_formatter_and_linter_tools_established,
                were_formatter_and_linter_tools_established_score
            ),
            (
                were_static_analysis_tools_established,
                were_static_analysis_tools_established_score
            ),
            (
                were_semi_automated_tools_configured,
                were_semi_automated_tools_configured_score
            ),
            (
                were_semi_automated_processes_run,
                were_semi_automated_processes_run_score
            ),
            (were_automated_tests_run, were_automated_tests_run_score),
            (
                were_tests_written_like_production_code,
                were_tests_written_like_production_code_score
            ),
            (
                were_unit_tests_written_with_boc,
                were_unit_tests_written_with_boc_score
            )
        );

        feedbacks_with_data.push(feedback_with_data);
    }

    let mut results: Vec<QuestionCorrelationData> = Vec::new();

    parse_ranks!(
        feedbacks_with_data,
        (
            were_requirements_documents_written_score,
            were_requirements_documents_written_rank
        ),
        (
            was_initial_project_plan_written_score,
            was_initial_project_plan_written_rank
        ),
        (
            were_specification_documents_written_score,
            were_specification_documents_written_rank
        ),
        (
            were_technical_design_documents_written_score,
            were_technical_design_documents_written_rank
        ),
        (
            was_sbom_document_written_score,
            was_sbom_document_written_rank
        ),
        (
            were_draft_design_documents_written_score,
            were_draft_design_documents_written_rank
        ),
        (
            were_project_documents_reviewed_score,
            were_project_documents_reviewed_rank
        ),
        (
            were_mutability_and_side_effects_avoided_score,
            were_mutability_and_side_effects_avoided_rank
        ),
        (
            were_mutable_names_encoded_score,
            were_mutable_names_encoded_rank
        ),
        (was_non_english_used_score, was_non_english_used_rank),
        (was_code_style_enforced_score, was_code_style_enforced_rank),
        (was_code_refactored_score, was_code_refactored_rank),
        (
            were_posix_timestamps_used_score,
            were_posix_timestamps_used_rank
        ),
        (
            were_magic_numbers_replaced_score,
            were_magic_numbers_replaced_rank
        ),
        (
            were_critical_code_commented_score,
            were_critical_code_commented_rank
        ),
        (
            were_regex_patterns_commented_score,
            were_regex_patterns_commented_rank
        ),
        (
            were_todo_comments_avoided_score,
            were_todo_comments_avoided_rank
        ),
        (
            was_ai_generated_code_reviewed_score,
            was_ai_generated_code_reviewed_rank
        ),
        (
            was_branching_strategy_established_score,
            was_branching_strategy_established_rank
        ),
        (
            was_branch_naming_strategy_established_score,
            was_branch_naming_strategy_established_rank
        ),
        (
            was_commit_message_format_established_score,
            was_commit_message_format_established_rank
        ),
        (
            was_merging_strategy_established_score,
            was_merging_strategy_established_rank
        ),
        (
            were_higher_level_design_issues_considered_score,
            were_higher_level_design_issues_considered_rank
        ),
        (was_reviewer_required_score, was_reviewer_required_rank),
        (
            were_formatter_and_linter_tools_established_score,
            were_formatter_and_linter_tools_established_rank
        ),
        (
            were_static_analysis_tools_established_score,
            were_static_analysis_tools_established_rank
        ),
        (
            were_semi_automated_tools_configured_score,
            were_semi_automated_tools_configured_rank
        ),
        (
            were_semi_automated_processes_run_score,
            were_semi_automated_processes_run_rank
        ),
        (
            were_automated_tests_run_score,
            were_automated_tests_run_rank
        ),
        (
            were_tests_written_like_production_code_score,
            were_tests_written_like_production_code_rank
        ),
        (
            were_unit_tests_written_with_boc_score,
            were_unit_tests_written_with_boc_rank
        )
    );

    parse_correlations!(
        feedbacks_with_data,
        &mut results,
        (
            were_requirements_documents_written,
            were_requirements_documents_written_score,
            were_requirements_documents_written_rank,
            were_requirements_documents_written_spearman,
            were_requirements_documents_written_p_value
        ),
        (
            was_initial_project_plan_written,
            was_initial_project_plan_written_score,
            was_initial_project_plan_written_rank,
            was_initial_project_plan_written_spearman,
            was_initial_project_plan_written_p_value
        ),
        (
            were_specification_documents_written,
            were_specification_documents_written_score,
            were_specification_documents_written_rank,
            were_specification_documents_written_spearman,
            were_specification_documents_written_p_value
        ),
        (
            were_technical_design_documents_written,
            were_technical_design_documents_written_score,
            were_technical_design_documents_written_rank,
            were_technical_design_documents_written_spearman,
            were_technical_design_documents_written_p_value
        ),
        (
            was_sbom_document_written,
            was_sbom_document_written_score,
            was_sbom_document_written_rank,
            was_sbom_document_written_spearman,
            was_sbom_document_written_p_value
        ),
        (
            were_draft_design_documents_written,
            were_draft_design_documents_written_score,
            were_draft_design_documents_written_rank,
            were_draft_design_documents_written_spearman,
            were_draft_design_documents_written_p_value
        ),
        (
            were_project_documents_reviewed,
            were_project_documents_reviewed_score,
            were_project_documents_reviewed_rank,
            were_project_documents_reviewed_spearman,
            were_project_documents_reviewed_p_value
        ),
        (
            were_mutability_and_side_effects_avoided,
            were_mutability_and_side_effects_avoided_score,
            were_mutability_and_side_effects_avoided_rank,
            were_mutability_and_side_effects_avoided_spearman,
            were_mutability_and_side_effects_avoided_p_value
        ),
        (
            were_mutable_names_encoded,
            were_mutable_names_encoded_score,
            were_mutable_names_encoded_rank,
            were_mutable_names_encoded_spearman,
            were_mutable_names_encoded_p_value
        ),
        (
            was_non_english_used,
            was_non_english_used_score,
            was_non_english_used_rank,
            was_non_english_used_spearman,
            was_non_english_used_p_value
        ),
        (
            was_code_style_enforced,
            was_code_style_enforced_score,
            was_code_style_enforced_rank,
            was_code_style_enforced_spearman,
            was_code_style_enforced_p_value
        ),
        (
            was_code_refactored,
            was_code_refactored_score,
            was_code_refactored_rank,
            was_code_refactored_spearman,
            was_code_refactored_p_value
        ),
        (
            were_posix_timestamps_used,
            were_posix_timestamps_used_score,
            were_posix_timestamps_used_rank,
            were_posix_timestamps_used_spearman,
            were_posix_timestamps_used_p_value
        ),
        (
            were_magic_numbers_replaced,
            were_magic_numbers_replaced_score,
            were_magic_numbers_replaced_rank,
            were_magic_numbers_replaced_spearman,
            were_magic_numbers_replaced_p_value
        ),
        (
            were_critical_code_commented,
            were_critical_code_commented_score,
            were_critical_code_commented_rank,
            were_critical_code_commented_spearman,
            were_critical_code_commented_p_value
        ),
        (
            were_regex_patterns_commented,
            were_regex_patterns_commented_score,
            were_regex_patterns_commented_rank,
            were_regex_patterns_commented_spearman,
            were_regex_patterns_commented_p_value
        ),
        (
            were_todo_comments_avoided,
            were_todo_comments_avoided_score,
            were_todo_comments_avoided_rank,
            were_todo_comments_avoided_spearman,
            were_todo_comments_avoided_p_value
        ),
        (
            was_ai_generated_code_reviewed,
            was_ai_generated_code_reviewed_score,
            was_ai_generated_code_reviewed_rank,
            was_ai_generated_code_reviewed_spearman,
            was_ai_generated_code_reviewed_p_value
        ),
        (
            was_branching_strategy_established,
            was_branching_strategy_established_score,
            was_branching_strategy_established_rank,
            was_branching_strategy_established_spearman,
            was_branching_strategy_established_p_value
        ),
        (
            was_branch_naming_strategy_established,
            was_branch_naming_strategy_established_score,
            was_branch_naming_strategy_established_rank,
            was_branch_naming_strategy_established_spearman,
            was_branch_naming_strategy_established_p_value
        ),
        (
            was_commit_message_format_established,
            was_commit_message_format_established_score,
            was_commit_message_format_established_rank,
            was_commit_message_format_established_spearman,
            was_commit_message_format_established_p_value
        ),
        (
            was_merging_strategy_established,
            was_merging_strategy_established_score,
            was_merging_strategy_established_rank,
            was_merging_strategy_established_spearman,
            was_merging_strategy_established_p_value
        ),
        (
            were_higher_level_design_issues_considered,
            were_higher_level_design_issues_considered_score,
            were_higher_level_design_issues_considered_rank,
            were_higher_level_design_issues_considered_spearman,
            were_higher_level_design_issues_considered_p_value
        ),
        (
            was_reviewer_required,
            was_reviewer_required_score,
            was_reviewer_required_rank,
            was_reviewer_required_spearman,
            was_reviewer_required_p_value
        ),
        (
            were_formatter_and_linter_tools_established,
            were_formatter_and_linter_tools_established_score,
            were_formatter_and_linter_tools_established_rank,
            were_formatter_and_linter_tools_established_spearman,
            were_formatter_and_linter_tools_established_p_value
        ),
        (
            were_static_analysis_tools_established,
            were_static_analysis_tools_established_score,
            were_static_analysis_tools_established_rank,
            were_static_analysis_tools_established_spearman,
            were_static_analysis_tools_established_p_value
        ),
        (
            were_semi_automated_tools_configured,
            were_semi_automated_tools_configured_score,
            were_semi_automated_tools_configured_rank,
            were_semi_automated_tools_configured_spearman,
            were_semi_automated_tools_configured_p_value
        ),
        (
            were_semi_automated_processes_run,
            were_semi_automated_processes_run_score,
            were_semi_automated_processes_run_rank,
            were_semi_automated_processes_run_spearman,
            were_semi_automated_processes_run_p_value
        ),
        (
            were_automated_tests_run,
            were_automated_tests_run_score,
            were_automated_tests_run_rank,
            were_automated_tests_run_spearman,
            were_automated_tests_run_p_value
        ),
        (
            were_tests_written_like_production_code,
            were_tests_written_like_production_code_score,
            were_tests_written_like_production_code_rank,
            were_tests_written_like_production_code_spearman,
            were_tests_written_like_production_code_p_value
        ),
        (
            were_unit_tests_written_with_boc,
            were_unit_tests_written_with_boc_score,
            were_unit_tests_written_with_boc_rank,
            were_unit_tests_written_with_boc_spearman,
            were_unit_tests_written_with_boc_p_value
        )
    );

    results
}

pub fn get_p_value(t: f64, n: usize) -> f64 {
    let t_dist = StudentsT::new(0f64, 1f64, (n - 2) as f64)
        .expect("Failed to create StudentsT distribution");

    let p_value = 1f64 - t_dist.cdf(t.abs());
    return p_value;
}
