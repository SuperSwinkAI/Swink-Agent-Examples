//! Example: deep-research multi-agent pipeline with TUI and artifact persistence.
//!
//! The user types a research topic in the TUI. A coordinator agent drives the
//! full pipeline step-by-step, with each phase visible as a distinct tool call:
//!
//! 1. `orchestrate_research` — breaks the topic into sub-questions.
//! 2. `research_question`    — researches one question and saves findings
//!                             as `research/qN.md` via `SaveArtifactTool`.
//! 3. `synthesize_and_save`  — loads all research artifacts, synthesizes a
//!                             report, and saves it to the file path the user
//!                             (or agent) specifies.
//!
//! The TUI shows each tool call as it happens, so the user sees live progress
//! through the pipeline.  All agents share one `Arc<InMemoryArtifactStore>`
//! and a fixed `SESSION_ID` so artifacts are visible across phases.
//!
//! # Run
//!
//! ```text
//! cargo run
//! ```
//!
//! Then type a research topic and press Enter.
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY` in environment or `.env` file

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;

use swink_agent::{
    Agent, AgentOptions, ApprovalMode, FnTool, IntoTool, ModelConnections, SaveArtifactTool,
    artifact_tools,
    artifact::{
        ArtifactData, ArtifactError, ArtifactMeta, ArtifactStore, ArtifactVersion,
        validate_artifact_name,
    },
};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_tui::{TuiConfig, launch, restore_terminal, setup_terminal};

const SESSION_ID: &str = "deep-research-session";
const MODEL: &str = "claude-sonnet-4-6";

// ─── In-memory artifact store ─────────────────────────────────────────────────

type VersionEntry = (ArtifactVersion, ArtifactData);

/// Simple in-memory `ArtifactStore` shared across all pipeline agents via `Arc`.
struct InMemoryArtifactStore {
    data: Mutex<HashMap<String, HashMap<String, Vec<VersionEntry>>>>,
}

impl InMemoryArtifactStore {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl ArtifactStore for InMemoryArtifactStore {
    async fn save(
        &self,
        session_id: &str,
        name: &str,
        data: ArtifactData,
    ) -> Result<ArtifactVersion, ArtifactError> {
        validate_artifact_name(name)?;
        let mut guard = self.data.lock().await;
        let session = guard.entry(session_id.to_string()).or_default();
        let versions = session.entry(name.to_string()).or_default();
        #[allow(clippy::cast_possible_truncation)]
        let version_num = versions.len() as u32 + 1;
        let version = ArtifactVersion {
            name: name.to_string(),
            version: version_num,
            created_at: Utc::now(),
            size: data.content.len(),
            content_type: data.content_type.clone(),
        };
        versions.push((version.clone(), data));
        Ok(version)
    }

    async fn load(
        &self,
        session_id: &str,
        name: &str,
    ) -> Result<Option<(ArtifactData, ArtifactVersion)>, ArtifactError> {
        let guard = self.data.lock().await;
        Ok(guard
            .get(session_id)
            .and_then(|s| s.get(name))
            .and_then(|v| v.last())
            .map(|(ver, data)| (data.clone(), ver.clone())))
    }

    async fn load_version(
        &self,
        session_id: &str,
        name: &str,
        version: u32,
    ) -> Result<Option<(ArtifactData, ArtifactVersion)>, ArtifactError> {
        let guard = self.data.lock().await;
        Ok(guard
            .get(session_id)
            .and_then(|s| s.get(name))
            .and_then(|v| v.iter().find(|(ver, _)| ver.version == version))
            .map(|(ver, data)| (data.clone(), ver.clone())))
    }

    async fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>, ArtifactError> {
        let guard = self.data.lock().await;
        let Some(session) = guard.get(session_id) else {
            return Ok(vec![]);
        };
        let metas = session
            .iter()
            .filter_map(|(name, versions)| {
                let first = versions.first()?;
                let last = versions.last()?;
                Some(ArtifactMeta {
                    name: name.clone(),
                    latest_version: last.0.version,
                    created_at: first.0.created_at,
                    updated_at: last.0.created_at,
                    content_type: last.0.content_type.clone(),
                })
            })
            .collect();
        Ok(metas)
    }

    async fn delete(&self, session_id: &str, name: &str) -> Result<(), ArtifactError> {
        let mut guard = self.data.lock().await;
        if let Some(session) = guard.get_mut(session_id) {
            session.remove(name);
        }
        Ok(())
    }
}

// ─── Tool parameter types ─────────────────────────────────────────────────────

// These structs are used only for JSON schema generation via `with_schema_for::<T>()`.
// The actual values are extracted from the raw `serde_json::Value` params at runtime.
#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct OrchestrateParams {
    /// The research topic to break into sub-questions.
    topic: String,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct ResearchQuestionParams {
    /// The question to research.
    question: String,
    /// Artifact name to save findings to (e.g. "research/q1.md").
    artifact_name: String,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct SynthesizeParams {
    /// File path where the final report will be written (e.g. "report.md").
    report_path: String,
}

// ─── Tool builders ────────────────────────────────────────────────────────────

/// Tool 1: run the orchestrator agent and return sub-questions.
fn make_orchestrate_tool(connection: swink_agent::ModelConnection) -> impl IntoTool {
    FnTool::new(
        "orchestrate_research",
        "Orchestrate Research",
        "Break a research topic into focused sub-questions. Returns a numbered list \
         of sub-questions to investigate.",
    )
    .with_schema_for::<OrchestrateParams>()
    .with_execute_simple(move |params, _cancel| {
        let conn = connection.clone();
        async move {
            let topic = match params["topic"].as_str() {
                Some(t) => t.to_string(),
                None => return swink_agent::AgentToolResult::error("missing topic"),
            };

            let options = AgentOptions::from_connections(
                "You are a research orchestrator. Given a research topic, identify exactly 3 \
                 focused sub-questions that together provide comprehensive coverage. \
                 Output each question on its own line, prefixed exactly with 'QUESTION: '. \
                 Output nothing else — no intro, no commentary, just the three QUESTION lines.",
                ModelConnections::new(conn, vec![]),
            );

            let mut agent = Agent::new(options);
            match agent.prompt_text(format!("Research topic: {topic}")).await {
                Ok(result) => {
                    let text = result.assistant_text();
                    let questions: Vec<&str> = text
                        .lines()
                        .filter_map(|l| l.trim().strip_prefix("QUESTION: "))
                        .collect();

                    if questions.is_empty() {
                        swink_agent::AgentToolResult::error(
                            "Orchestrator produced no questions. Try rephrasing the topic.",
                        )
                    } else {
                        let numbered = questions
                            .iter()
                            .enumerate()
                            .map(|(i, q)| format!("{}. {q}", i + 1))
                            .collect::<Vec<_>>()
                            .join("\n");
                        swink_agent::AgentToolResult::text(format!(
                            "Sub-questions for '{topic}':\n{numbered}"
                        ))
                    }
                }
                Err(e) => swink_agent::AgentToolResult::error(format!("orchestrator error: {e}")),
            }
        }
    })
}

/// Tool 2: run one researcher agent and save findings as an artifact.
fn make_research_question_tool(
    connection: swink_agent::ModelConnection,
    store: Arc<InMemoryArtifactStore>,
) -> impl IntoTool {
    FnTool::new(
        "research_question",
        "Research Question",
        "Research a single question and save detailed findings as a named artifact. \
         Call this once per sub-question returned by orchestrate_research.",
    )
    .with_schema_for::<ResearchQuestionParams>()
    .with_execute_simple(move |params, _cancel| {
        let conn = connection.clone();
        let store = Arc::clone(&store);
        async move {
            let question = match params["question"].as_str() {
                Some(q) => q.to_string(),
                None => return swink_agent::AgentToolResult::error("missing question"),
            };
            let artifact_name = match params["artifact_name"].as_str() {
                Some(n) => n.to_string(),
                None => return swink_agent::AgentToolResult::error("missing artifact_name"),
            };

            let options = AgentOptions::from_connections(
                "You are a research specialist. When given a question, research it thoroughly \
                 using your knowledge and save your complete, detailed findings using \
                 save_artifact. Structure findings with: a summary, key points, supporting \
                 evidence or examples, and implications.",
                ModelConnections::new(conn, vec![]),
            )
            .with_tools(vec![SaveArtifactTool::new(store).into_tool()])
            .with_state_entry("session_id", SESSION_ID);

            let mut agent = Agent::new(options);
            match agent
                .prompt_text(format!(
                    "Research the following question thoroughly and save your complete findings \
                     as artifact '{artifact_name}' with content_type 'text/markdown'.\n\n\
                     Question: {question}"
                ))
                .await
            {
                Ok(_) => swink_agent::AgentToolResult::text(format!(
                    "Research complete. Findings saved to artifact '{artifact_name}'."
                )),
                Err(e) => swink_agent::AgentToolResult::error(format!("researcher error: {e}")),
            }
        }
    })
}

/// Tool 3: synthesize all research artifacts into a file.
fn make_synthesize_tool(
    connection: swink_agent::ModelConnection,
    store: Arc<InMemoryArtifactStore>,
) -> impl IntoTool {
    FnTool::new(
        "synthesize_and_save",
        "Synthesize & Save Report",
        "Load all research artifacts, synthesize a comprehensive report, and write it \
         to a local file. Returns the path of the saved file.",
    )
    .with_schema_for::<SynthesizeParams>()
    .with_execute_simple(move |params, _cancel| {
        let conn = connection.clone();
        let store = Arc::clone(&store);
        async move {
            let report_path = match params["report_path"].as_str() {
                Some(p) => p.to_string(),
                None => return swink_agent::AgentToolResult::error("missing report_path"),
            };

            // Run the synthesizer agent — it uses list/load/save artifact tools
            // to gather all research and produce report/final.md.
            let options = AgentOptions::from_connections(
                "You are a research synthesizer. Follow these steps:\n\
                 1. Call list_artifacts to see all available research.\n\
                 2. Call load_artifact for each research artifact.\n\
                 3. Write a comprehensive, well-structured report integrating all findings.\n\
                 4. Save the report as artifact 'report/final.md' with content_type \
                    'text/markdown'.\n\
                 The report must have: an executive summary, detailed findings per \
                 sub-question, a synthesis section, and a conclusion.",
                ModelConnections::new(conn, vec![]),
            )
            .with_tools(artifact_tools(Arc::clone(&store)))
            .with_state_entry("session_id", SESSION_ID);

            let mut agent = Agent::new(options);
            let prompt_result = agent
                .prompt_text(
                    "Use the artifact tools to load all research findings, then synthesize \
                     and save a final integrated report as 'report/final.md'."
                        .to_string(),
                )
                .await;

            if let Err(e) = prompt_result {
                return swink_agent::AgentToolResult::error(format!("synthesizer error: {e}"));
            }

            // Load the synthesized report from the artifact store and write to disk.
            match store.load(SESSION_ID, "report/final.md").await {
                Ok(Some((data, version))) => {
                    let content = String::from_utf8_lossy(&data.content).into_owned();
                    match std::fs::write(&report_path, &content) {
                        Ok(()) => swink_agent::AgentToolResult::text(format!(
                            "Report saved to `{report_path}` ({} bytes, artifact v{}).",
                            version.size, version.version
                        )),
                        Err(e) => swink_agent::AgentToolResult::error(format!(
                            "Could not write file '{report_path}': {e}"
                        )),
                    }
                }
                Ok(None) => swink_agent::AgentToolResult::error(
                    "Synthesizer did not produce 'report/final.md'. \
                     Try running synthesize_and_save again.",
                ),
                Err(e) => swink_agent::AgentToolResult::error(format!("artifact store error: {e}")),
            }
        }
    })
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model(MODEL)?;
    let store: Arc<InMemoryArtifactStore> = Arc::new(InMemoryArtifactStore::new());

    // Build the three pipeline tools, each capturing a cloned connection and
    // Arc reference to the shared store.
    let orchestrate = make_orchestrate_tool(connection.clone()).into_tool();
    let research = make_research_question_tool(connection.clone(), Arc::clone(&store)).into_tool();
    let synthesize = make_synthesize_tool(connection.clone(), Arc::clone(&store)).into_tool();

    // The coordinator agent drives the pipeline via tool calls.  ApprovalMode::Bypassed
    // means the agent never pauses to ask for tool approval — the full pipeline runs
    // autonomously once the user submits their research topic.
    let options = AgentOptions::from_connections(
        "You are a deep research coordinator. When the user gives you a research topic:\n\
         1. Call orchestrate_research with the topic to get sub-questions.\n\
         2. Call research_question for EACH sub-question. Use artifact names like \
            'research/q1.md', 'research/q2.md', etc. (one per question).\n\
         3. Call synthesize_and_save with a descriptive file name (e.g. 'report.md') \
            to generate and write the final report.\n\
         4. Tell the user the report is ready and where to find it.\n\n\
         Always follow all four steps in order. Do not skip any step.",
        ModelConnections::new(connection, vec![]),
    )
    .with_tools(vec![orchestrate, research, synthesize])
    .with_approval_mode(ApprovalMode::Bypassed);

    let mut terminal = setup_terminal()?;
    let result = launch(TuiConfig::load(), &mut terminal, options).await;
    restore_terminal()?;

    result
}
