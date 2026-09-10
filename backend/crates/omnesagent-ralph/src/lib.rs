//! Ralph Orchestrator: Autonomous development cycle (Ralph Loop x OpenSpec x Ponytail)
//! for Omnes Agent.
//!
//! Conforms to specification `Ralph Orchestrator (Omnes Agent).md` (v2.1).

pub mod approval;
pub mod debt;
pub mod events;
pub mod executor;
pub mod git;
pub mod journal;
pub mod knowledge;
pub mod loop_driver;
pub mod minimality;
pub mod openspec;
pub mod plateau;
pub mod state;
pub mod tasks;

use approval::GateManager;
use events::RalphEvent;
use executor::{Executor, SubprocessCliExecutor};
use knowledge::RalphKnowledgeService;
use omnesagent_config::ralph::RalphConfig;
use openspec::OpenSpecWorkspace;
use state::{RalphPhase, RalphState, StateError};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum RalphError {
    #[error("An active run already exists for slug '{0}': ERR_RUN_ACTIVE")]
    RunActive(String),
    #[error("Run for slug '{0}' not found")]
    RunNotFound(String),
    #[error("State error: {0}")]
    State(#[from] StateError),
    #[error("Driver error: {0}")]
    Driver(#[from] loop_driver::DriverError),
    #[error("Tasks error: {0}")]
    Tasks(#[from] tasks::TasksError),
    #[error("OpenSpec error: {0}")]
    OpenSpec(#[from] openspec::OpenSpecError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Handle to an actively running or managed Ralph run.
pub struct RunHandle {
    pub run_id: String,
    pub slug: String,
    pub stop_signal: Arc<AtomicBool>,
}

/// Central orchestrator instance managing Ralph runs across the workspace.
pub struct RalphOrchestrator {
    pub repo_root: PathBuf,
    pub config: RalphConfig,
    pub knowledge: RalphKnowledgeService,
    pub gate_mgr: Arc<GateManager>,
    pub runs: Arc<RwLock<HashMap<String, RunHandle>>>,
    pub event_tx: Sender<RalphEvent>,
}

impl RalphOrchestrator {
    pub fn new(
        repo_root: PathBuf,
        config: RalphConfig,
        knowledge: RalphKnowledgeService,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            repo_root,
            config,
            knowledge,
            gate_mgr: Arc::new(GateManager::new()),
            runs: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Subscribe to broadcast events from Ralph runs.
    pub fn subscribe(&self) -> Receiver<RalphEvent> {
        self.event_tx.subscribe()
    }

    /// Start or resume a Ralph run on the specified feature slug.
    pub async fn start_run(
        &self,
        slug: &str,
        custom_executor: Option<Arc<dyn Executor>>,
    ) -> Result<String, RalphError> {
        let ws = OpenSpecWorkspace::new(&self.repo_root, slug);
        ws.init()?;

        // Concurrency guard (§5.5)
        {
            let runs_guard = self.runs.read().await;
            if runs_guard.contains_key(slug) {
                return Err(RalphError::RunActive(slug.to_string()));
            }
        }
        RalphState::check_concurrency(&ws.state_path(), slug)?;

        let stop_signal = Arc::new(AtomicBool::new(false));
        let run_id = format!("run_{}", chrono::Utc::now().timestamp_millis());

        let handle = RunHandle {
            run_id: run_id.clone(),
            slug: slug.to_string(),
            stop_signal: stop_signal.clone(),
        };

        {
            let mut runs_guard = self.runs.write().await;
            runs_guard.insert(slug.to_string(), handle);
        }

        // Setup executor
        let executor: Arc<dyn Executor> = if let Some(exec) = custom_executor {
            exec
        } else {
            let cmd = if !self.config.executor_command.is_empty() {
                self.config.executor_command.clone()
            } else {
                format!("{:?}", self.config.executor).to_lowercase()
            };
            Arc::new(SubprocessCliExecutor::new(
                cmd,
                self.config.executor_timeout_secs,
                self.repo_root.clone(),
            ))
        };

        let repo_root = self.repo_root.clone();
        let slug_clone = slug.to_string();
        let config = self.config.clone();
        let knowledge = self.knowledge.clone();
        let gate_mgr = self.gate_mgr.clone();
        let event_tx = Some(self.event_tx.clone());
        let runs_map = self.runs.clone();

        // Spawn async background driver task using omnesagent-spawn
        omnesagent_spawn::spawn!(async move {
            let res = loop_driver::drive_loop(
                repo_root,
                slug_clone.clone(),
                config,
                executor,
                knowledge,
                gate_mgr,
                event_tx,
                stop_signal,
            )
            .await;

            if let Err(e) = res {
                eprintln!("Ralph run for slug '{}' failed: {}", slug_clone, e);
            }

            let mut runs_guard = runs_map.write().await;
            runs_guard.remove(&slug_clone);
        });

        Ok(run_id)
    }

    /// Signal a running cycle to gracefully stop.
    pub async fn stop_run(&self, slug: &str) -> Result<(), RalphError> {
        let runs_guard = self.runs.read().await;
        if let Some(handle) = runs_guard.get(slug) {
            handle.stop_signal.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            // If not active in-memory, check state.json and mark stopped directly
            let ws = OpenSpecWorkspace::new(&self.repo_root, slug);
            if ws.state_path().exists() {
                let mut state = RalphState::read_from(&ws.state_path())?;
                if !state.phase.is_terminal() {
                    state.transition_to(RalphPhase::Stopped)?;
                    state.write_atomic(&ws.state_path())?;
                }
                Ok(())
            } else {
                Err(RalphError::RunNotFound(slug.to_string()))
            }
        }
    }

    /// Read the current state of a feature run.
    pub fn get_status(&self, slug: &str) -> Result<RalphState, RalphError> {
        let ws = OpenSpecWorkspace::new(&self.repo_root, slug);
        if !ws.state_path().exists() {
            return Err(RalphError::RunNotFound(slug.to_string()));
        }
        let state = RalphState::read_from(&ws.state_path())?;
        Ok(state)
    }
}
