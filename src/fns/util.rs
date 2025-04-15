use crate::util::{
    types::Feedback,
    vars::{self},
};

pub fn assign_ids(feedbacks: &mut Vec<Feedback>) {
    for (i, feedback) in feedbacks.iter_mut().enumerate() {
        feedback.id = Some(i);
    }
}

pub fn get_answer_score(answer: &str) -> i8 {
    // The weightings were werified to be valid for each question despite overlaps in answers.
    return match answer {
        "Yes, with major refactoring done in separate branches" => vars::WEIGHTING_FULL,
        "Yes, variables with meaningful names were created" => vars::WEIGHTING_FULL,
        "Yes, all or most configurations were centralised" => vars::WEIGHTING_FULL,
        "Yes, all or most tests were run periodically" => vars::WEIGHTING_FULL,
        "Yes, all or most documents were reviewed" => vars::WEIGHTING_FULL,
        "Yes, they were written and maintained" => vars::WEIGHTING_FULL,
        "Yes, it was written and maintained" => vars::WEIGHTING_FULL,
        "Yes, established and enforced" => vars::WEIGHTING_FULL,
        "No, mostly English was used" => vars::WEIGHTING_FULL,
        "Yes, all or most tools" => vars::WEIGHTING_FULL,
        "Yes, always or often" => vars::WEIGHTING_FULL,
        "Yes" => vars::WEIGHTING_FULL,

        "Partially; they were written but not maintained" => vars::WEIGHTING_PARTIAL,
        "Partially; it was written but not maintained" => vars::WEIGHTING_PARTIAL,
        "Partially; some tests were run periodically" => vars::WEIGHTING_PARTIAL,
        "Yes, some configurations were centralised" => vars::WEIGHTING_PARTIAL,
        "Partially; established but not enforced" => vars::WEIGHTING_PARTIAL,
        "Partially; some documents were reviewed" => vars::WEIGHTING_PARTIAL,
        "Yes, without separate branches" => vars::WEIGHTING_PARTIAL,
        "Yes, some tools" => vars::WEIGHTING_PARTIAL,
        "Yes, sometimes" => vars::WEIGHTING_PARTIAL,

        "No, they were purposefully not avoided or this practice was not considered" => {
            vars::WEIGHTING_FULL_NEG
        }
        "No, \"magic numbers\" were used as literal values" => vars::WEIGHTING_FULL_NEG,
        "Yes, another language was used a lot" => vars::WEIGHTING_FULL_NEG,
        "No" => vars::WEIGHTING_FULL_NEG,

        "Yes, another language was used to some degree" => vars::WEIGHTING_PARTIAL_NEG,

        "I don't know (e.g. you don't know how your team members use generative AI tools)" => {
            vars::WEIGHTING_NOT_APPLICABLE
        }
        "AI tools were not used to generate code" => vars::WEIGHTING_NOT_APPLICABLE,
        "Regex patterns were not used" => vars::WEIGHTING_NOT_APPLICABLE,
        "Not applicable" => vars::WEIGHTING_NOT_APPLICABLE,
        "I don't know" => vars::WEIGHTING_NOT_APPLICABLE,
        _ => panic!("Invalid answer: {}", answer),
    };
}
