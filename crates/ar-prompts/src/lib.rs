//! Prompt templates and JSON schema for the review pipeline.
//!
//! Two responsibilities:
//! 1. Provide the strict JSON Schema the LLM emits its review against, plus a
//!    typed [`ReviewOutput`] for downstream activities.
//! 2. Render the user prompt that ships diff + PR context to the LLM.

pub mod fence;
pub mod prompt;
pub mod schema;
pub mod types;
pub mod validate;
pub mod verification;

#[expect(
    clippy::useless_attribute,
    reason = "clippy flags any lint attribute on a use item outside its fixed list, which omits \
              pub_use, yet the pub_use expectation below is what suppresses that lint here"
)]
#[expect(
    clippy::pub_use,
    reason = "ar-review (heal, error, mapping, pipeline, review_core, verify, red_team_pipeline) \
              and ar-cli (bench, commands) import these items from the crate root; dropping the \
              re-exports breaks them"
)]
pub use crate::{
    fence::{fence_nonce, fence_untrusted, process_nonce_salt, FENCE_MARKER_PREFIX},
    prompt::{
        japanese_output_enabled, parse_japanese_output, render_review_prompt, system_prompt,
        system_prompt_for, ReviewPromptInputs, JAPANESE_OUTPUT_INSTRUCTION, PR_BODY_MAX_BYTES,
        PR_TITLE_MAX_BYTES,
    },
    schema::review_schema,
    types::{FindingCategory, ReviewFinding, ReviewOutput, ReviewSeverity},
    validate::{validate_review_output, ValidationError},
    verification::{
        validate_verification_output, verification_schema, verification_system_prompt,
        VerificationOutput, VerificationValidationError, VerificationVerdict,
    },
};
