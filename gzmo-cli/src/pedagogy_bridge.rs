//! Chat integration helpers for the Agentic Teacher stack.

use std::path::PathBuf;

use anyhow::Result;

use gzmo_core::config::{GzmoConfig, PedagogyDefaultMode, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmGateway};
use gzmo_core::mentor_client::MentorResponse;
use gzmo_core::pedagogy::{
    classify_intent, InteractionIntent, LearnerProfile, LearnerStore, OrchestratorInput,
    OrchestratorOutput, PedagogyOrchestrator, PedagogySession, PrerequisiteGraph,
};
use gzmo_core::types::{Message, Role};

const DELEGATE_HINT_OPS_MODE: &str = "Ops mode active. Run the request with bash/shell tools; \
    do not call gzmo_mentor_teach until /ops toggles mentor back.";

const DELEGATE_HINT_OPS_INTENT: &str =
    "Ops intent detected. Run the request with bash/shell tools, \
    or toggle /ops for sustained execution mode.";

/// True when the client should execute locally instead of running the Socratic orchestrator.
pub fn should_delegate_exec(session: &PedagogySession, input: &str) -> bool {
    if session.ops_mode {
        return true;
    }
    classify_intent(
        input,
        false,
        session.learn_prep_topic.is_some(),
        session.learn_prep_notes.is_some(),
    ) == InteractionIntent::Ops
}

pub fn delegate_exec_response(
    message: &str,
    session: &PedagogySession,
    learner_id: &str,
) -> MentorResponse {
    let hint = if session.ops_mode {
        DELEGATE_HINT_OPS_MODE
    } else {
        DELEGATE_HINT_OPS_INTENT
    };
    MentorResponse::delegate_exec(message, session.ops_mode, learner_id.to_string(), hint)
}

pub struct PedagogyRuntime {
    pub orchestrator: PedagogyOrchestrator,
    pub learner_store: LearnerStore,
    pub learner_profile: LearnerProfile,
    pub session: PedagogySession,
}

impl PedagogyRuntime {
    pub async fn boot(config: &GzmoConfig) -> Result<Self> {
        let pedagogy = &config.pedagogy;
        let learner_store = LearnerStore::new(pedagogy);
        let learner_profile = learner_store.load().await?;
        let mut session = PedagogySession::load(pedagogy).await?;
        if pedagogy.default_mode == PedagogyDefaultMode::Ops {
            session.ops_mode = true;
        }

        let graphs_dir = PathBuf::from(&pedagogy.prerequisite_graphs_dir);
        let graph = if graphs_dir.is_dir() {
            PrerequisiteGraph::load_dir(&graphs_dir).ok()
        } else {
            None
        };

        let orchestrator = PedagogyOrchestrator::new(pedagogy.clone(), graph);

        Ok(Self {
            orchestrator,
            learner_store,
            learner_profile,
            session,
        })
    }

    pub fn learner_prompt_suffix(&self) -> String {
        if self.session.ops_mode {
            String::new()
        } else {
            self.learner_profile.prompt_block(1200)
        }
    }

    pub fn conversation_tail(messages: &[Message], max_turns: usize) -> String {
        messages
            .iter()
            .rev()
            .filter(|m| !m.is_meta && matches!(m.role, Role::User | Role::Assistant))
            .take(max_turns * 2)
            .map(|m| {
                let label = if m.role == Role::User {
                    "Student"
                } else {
                    "GZMO"
                };
                format!("{label}: {}", m.content)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub async fn maybe_teach(
        &mut self,
        config: &GzmoConfig,
        router: &GatewayRouter,
        tutor_gateway: &dyn LlmGateway,
        input: &str,
        messages: &[Message],
        chaos_context: Option<&str>,
        discovery_context: Option<&str>,
        chaos_snapshot_rx: Option<&tokio::sync::watch::Receiver<gzmo_chaos::pulse::ChaosSnapshot>>,
    ) -> Result<Option<OrchestratorOutput>> {
        let internal_gateway = router.gateway(TaskKind::PedagogyInternal);
        if !config.pedagogy.enabled {
            return Ok(None);
        }

        let learn_prep_active = self.session.learn_prep_topic.is_some();
        let learn_prep_ready = self.session.learn_prep_notes.is_some();
        let intent = classify_intent(
            input,
            self.session.ops_mode,
            learn_prep_active,
            learn_prep_ready,
        );

        match intent {
            InteractionIntent::Ops => return Ok(None),
            InteractionIntent::LearnPrep => {
                if input.trim().starts_with("/learn ") {
                    return Ok(None);
                }
                if let Some(topic) = self.session.learn_prep_topic.clone() {
                    if self.session.learn_prep_notes.is_none() {
                        let prep = self
                            .orchestrator
                            .run_learn_prep(internal_gateway.as_ref(), &topic)
                            .await?;
                        self.session.learn_prep_notes = Some(prep);
                        self.session.save(&config.pedagogy).await?;
                    }
                }
            }
            InteractionIntent::Teach | InteractionIntent::LearnSync => {}
        }

        let was_awaiting_teachback = self.session.awaiting_teachback;
        if was_awaiting_teachback && input.trim().len() > 60 {
            self.learner_profile.record_teachback(input);
            self.session.awaiting_teachback = false;
        }

        let teachback_due = config.pedagogy.teachback_interval > 0
            && !was_awaiting_teachback
            && self.session.turns_since_teachback >= config.pedagogy.teachback_interval;

        let tail = Self::conversation_tail(messages, 4);
        let prep_notes = self.session.learn_prep_notes.as_deref();
        let output = self
            .orchestrator
            .run(
                tutor_gateway,
                internal_gateway.as_ref(),
                OrchestratorInput {
                    user_message: input,
                    learner_profile: &self.learner_profile,
                    trio_mode: self.session.trio_mode,
                    learn_prep_notes: prep_notes,
                    conversation_tail: &tail,
                    chaos_context,
                    discovery_context,
                    max_hint_level: config.pedagogy.max_hint_level,
                    teachback_due,
                    chaos_snapshot_rx,
                },
            )
            .await?;

        let summary = gzmo_core::text_util::truncate_chars(&output.response, 200);
        self.learner_profile.record_episode(&summary, None, None);
        let _ = self.learner_store.append_episode_markdown(&summary).await;
        self.learner_store.save(&self.learner_profile).await?;

        if teachback_due {
            self.session.awaiting_teachback = true;
            self.session.turns_since_teachback = 0;
        } else if !was_awaiting_teachback {
            self.session.turns_since_teachback += 1;
        }
        self.session.save(&config.pedagogy).await?;

        Ok(Some(output))
    }

    /// Reload session + learner profile from disk (after `/ops`, `/learn`, etc.).
    pub async fn reload_from_disk(&mut self) -> Result<()> {
        self.session = PedagogySession::load(self.learner_store.pedagogy_config()).await?;
        self.learner_profile = self.learner_store.load().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_core::pedagogy::PedagogySession;

    #[test]
    fn should_delegate_when_ops_mode() {
        let session = PedagogySession {
            ops_mode: true,
            ..Default::default()
        };
        assert!(should_delegate_exec(&session, "what is a symlink?"));
    }

    #[test]
    fn should_delegate_on_ops_intent_phrase() {
        let session = PedagogySession::default();
        assert!(should_delegate_exec(&session, "run ls -la /tmp"));
    }

    #[test]
    fn should_not_delegate_socratic_question() {
        let session = PedagogySession::default();
        assert!(!should_delegate_exec(&session, "what is a symlink?"));
    }
}
