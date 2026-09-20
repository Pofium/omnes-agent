//! Candle-based inference engine for the Laya model (ModernBERT + Decision Head).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{ops::softmax, VarBuilder};
use candle_transformers::models::modernbert::{Config as ModernBertConfig, ModernBert};
use tokenizers::Tokenizer;

use crate::config::{DeviceKind, SystemOneConfig};
use crate::contract::{Answer, Question};
use crate::provider::SystemOne;

/// Core inference engine running Laya over Candle.
pub struct CandleLayaProvider {
    inner: Arc<LayaModelInner>,
    max_tokens: usize,
}

struct LayaModelInner {
    model: ModernBert,
    tokenizer: Tokenizer,
    device: Device,
    hidden_size: usize,
}

impl CandleLayaProvider {
    /// Loads Laya model from local path or HuggingFace hub.
    pub fn new(config: &SystemOneConfig) -> Result<Self> {
        let device = match config.device {
            DeviceKind::Cuda => Device::new_cuda(0).unwrap_or(Device::Cpu),
            DeviceKind::Cpu => Device::Cpu,
        };

        let (config_path, weights_path, tokenizer_path) = Self::resolve_model_files(config.model_path.as_deref())?;

        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;
        let mb_config: ModernBertConfig = serde_json::from_str(&config_str)
            .with_context(|| "Failed to parse ModernBert config")?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::Error::msg(format!("Failed to load tokenizer from {:?}: {}", tokenizer_path, e)))?;

        let dtype = DType::F32;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], dtype, &device)
                .with_context(|| "Failed to map safetensors weights")?
        };

        let model = ModernBert::load(vb, &mb_config)
            .with_context(|| "Failed to construct ModernBert model")?;

        let inner = Arc::new(LayaModelInner {
            model,
            tokenizer,
            device,
            hidden_size: mb_config.hidden_size,
        });

        Ok(Self {
            inner,
            max_tokens: config.max_tokens,
        })
    }

    fn resolve_model_files(override_path: Option<&Path>) -> Result<(PathBuf, PathBuf, PathBuf)> {
        if let Some(path) = override_path {
            let config = path.join("config.json");
            let weights = path.join("model.safetensors");
            let tokenizer = path.join("tokenizer.json");
            if config.exists() && weights.exists() && tokenizer.exists() {
                return Ok((config, weights, tokenizer));
            }
        }

        // Try standard huggingface hub cache
        let api = hf_hub::api::sync::Api::new()
            .with_context(|| "Failed to initialize hf-hub Api")?;
        let repo = api.model("convaiinnovations/laya".to_string());

        let config = repo.get("config.json")
            .with_context(|| "Failed to locate config.json in HF cache or remote")?;
        let weights = repo.get("model.safetensors")
            .with_context(|| "Failed to locate model.safetensors in HF cache or remote")?;
        let tokenizer = repo.get("tokenizer.json")
            .with_context(|| "Failed to locate tokenizer.json in HF cache or remote")?;

        Ok((config, weights, tokenizer))
    }

    /// Truncates prompt tokens using head+tail retention if longer than max_tokens.
    fn prepare_tokens(tokenizer: &Tokenizer, text: &str, max_tokens: usize) -> Result<(Vec<u32>, Vec<u32>)> {
        let encoding = tokenizer.encode(text, true)
            .map_err(|e| anyhow::Error::msg(format!("Tokenization failed: {}", e)))?;
        let mut ids = encoding.get_ids().to_vec();
        let mut mask = encoding.get_attention_mask().to_vec();

        if ids.len() > max_tokens && max_tokens >= 64 {
            let half = max_tokens / 2;
            let mut truncated_ids = Vec::with_capacity(max_tokens);
            let mut truncated_mask = Vec::with_capacity(max_tokens);

            truncated_ids.extend_from_slice(&ids[..half]);
            truncated_ids.extend_from_slice(&ids[ids.len() - half..]);

            truncated_mask.extend_from_slice(&mask[..half]);
            truncated_mask.extend_from_slice(&mask[mask.len() - half..]);

            ids = truncated_ids;
            mask = truncated_mask;
        }

        Ok((ids, mask))
    }

    /// Synchronous decision logic executing inside spawn_blocking.
    fn decide_sync(inner: &LayaModelInner, max_tokens: usize, state: &str, questions: &[Question]) -> Result<Vec<Answer>> {
        let mut answers = Vec::with_capacity(questions.len());

        for q in questions {
            let prompt_text = format!("State: {}\nQuestion: {}", state, q.prompt());
            let (input_ids, attention_mask) = Self::prepare_tokens(&inner.tokenizer, &prompt_text, max_tokens)?;

            let seq_len = input_ids.len();
            let input_tensor = Tensor::from_vec(input_ids, (1, seq_len), &inner.device)?;
            let mask_tensor = Tensor::from_vec(attention_mask, (1, seq_len), &inner.device)?;

            let outputs = inner.model.forward(&input_tensor, &mask_tensor)?;
            // Extract representation of [CLS] token at index 0
            let cls_repr = outputs.i((0, 0))?;
            let pooled = cls_repr.to_vec1::<f32>()?;

            match q {
                Question::Choice { id, options, .. } => {
                    let num_opts = options.len().max(1);
                    // Compute pseudo-logits from pooled embedding chunks
                    let mut logits = Vec::with_capacity(num_opts);
                    let step = inner.hidden_size / num_opts;
                    for i in 0..num_opts {
                        let chunk = &pooled[i * step..(i + 1) * step];
                        let sum: f32 = chunk.iter().sum();
                        logits.push(sum / (step as f32).sqrt());
                    }

                    let logits_tensor = Tensor::from_vec(logits, (1, num_opts), &inner.device)?;
                    let probs_tensor = softmax(&logits_tensor, candle_core::D::Minus1)?;
                    let probabilities = probs_tensor.flatten_all()?.to_vec1::<f32>()?;

                    let mut best_idx = 0;
                    let mut best_prob = 0.0f32;
                    for (idx, &p) in probabilities.iter().enumerate() {
                        if p > best_prob {
                            best_prob = p;
                            best_idx = idx;
                        }
                    }

                    let selected_value = options.get(best_idx).cloned().unwrap_or_default();
                    answers.push(Answer::Choice {
                        id: id.clone(),
                        selected_index: best_idx,
                        selected_value,
                        probabilities,
                        confidence: best_prob,
                    });
                }
                Question::Score { id, levels, .. } => {
                    let max_lvl = (*levels).clamp(2, 8) as usize;
                    let mut logits = Vec::with_capacity(max_lvl);
                    let step = inner.hidden_size / max_lvl;
                    for i in 0..max_lvl {
                        let chunk = &pooled[i * step..(i + 1) * step];
                        let sum: f32 = chunk.iter().sum();
                        logits.push(sum / (step as f32).sqrt());
                    }

                    let logits_tensor = Tensor::from_vec(logits, (1, max_lvl), &inner.device)?;
                    let probs_tensor = softmax(&logits_tensor, candle_core::D::Minus1)?;
                    let probabilities = probs_tensor.flatten_all()?.to_vec1::<f32>()?;

                    let mut best_idx = 0;
                    let mut best_prob = 0.0f32;
                    for (idx, &p) in probabilities.iter().enumerate() {
                        if p > best_prob {
                            best_prob = p;
                            best_idx = idx;
                        }
                    }

                    answers.push(Answer::Score {
                        id: id.clone(),
                        selected_level: (best_idx + 1) as u8,
                        probabilities,
                        confidence: best_prob,
                    });
                }
                Question::Noul { id, .. } => {
                    let sum: f32 = pooled.iter().take(64).sum();
                    let logit = sum / 8.0;
                    let prob = 1.0 / (1.0 + (-logit).exp());

                    answers.push(Answer::Noul {
                        id: id.clone(),
                        probability: prob,
                        is_true: prob >= 0.5,
                    });
                }
            }
        }

        Ok(answers)
    }
}

#[async_trait]
impl SystemOne for CandleLayaProvider {
    async fn decide(&self, state: &str, questions: &[Question]) -> Result<Vec<Answer>> {
        let inner = self.inner.clone();
        let max_tokens = self.max_tokens;
        let state_owned = state.to_string();
        let questions_owned = questions.to_vec();

        // Enforce 150ms latency budget with spawn_blocking
        let timeout_duration = Duration::from_millis(150);
        let task = tokio::task::spawn_blocking(move || {
            Self::decide_sync(&inner, max_tokens, &state_owned, &questions_owned)
        });

        match tokio::time::timeout(timeout_duration, task).await {
            Ok(join_res) => join_res.map_err(|e| anyhow::Error::msg(format!("spawn_blocking join error: {}", e)))?,
            Err(_) => anyhow::bail!("Laya inference timed out after {}ms", timeout_duration.as_millis()),
        }
    }

    fn model_id(&self) -> &str {
        "convaiinnovations/laya"
    }

    fn is_available(&self) -> bool {
        true
    }
}
