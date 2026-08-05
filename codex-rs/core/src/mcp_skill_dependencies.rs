//! Titanium: MCP skill dependency install is hard-disabled.
use tokio_util::sync::CancellationToken;

use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_core_skills::SkillMetadata;
use codex_mcp::ElicitationReviewerHandle;

pub(crate) async fn maybe_prompt_and_install_mcp_dependencies(
    _sess: &Session,
    _turn_context: &TurnContext,
    _cancellation_token: &CancellationToken,
    _mentioned_skills: &[SkillMetadata],
    _elicitation_reviewer: Option<ElicitationReviewerHandle>,
) {
}
