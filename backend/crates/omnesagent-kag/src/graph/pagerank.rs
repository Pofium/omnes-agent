//! Итеративный Personalized PageRank (Ф29.1 v1.3; переиспользуется памятью — Ф33.1 v1.4).
//!
//! Идея HippoRAG/2: single-step multi-hop без графовых СУБД — pure Rust поверх
//! списка рёбер (`graph_edges` для графа знаний, `memory_relations` для памяти). Модуль не
//! знает, откуда пришли рёбра: на вход — плоские `(from, to, weight)` с целочисленными
//! индексами узлов, на выход — распределение вероятностей по узлам.
//!
//! Инварианты:
//! * damping `d ∈ [0.5, 0.85]` (старт 0.85) — значение клампится, а не отвергается;
//! * не более [`MAX_ITER`] итераций, сходимость по L1-норме (`tol`);
//! * **degree-normalization**: вес ребра делится на суммарный вес исходящих рёбер узла —
//!   hub-защита, иначе PPR «залипает» на записи-хабе со 100+ рёбрами;
//! * висячие узлы (без исходящих рёбер) не теряют массу — она возвращается в
//!   персонализацию (teleport), как в стандартном PPR;
//! * **usage-weighting** (столп III ob2h v2.0, [`personalized_pagerank_weighted`]):
//!   входящие рёбра узла взвешиваются [`usage_weight`] (`access_count`, recency-decay,
//!   trust, grace-period) — фундамент usage-weighted выдачи;
//! * никаких новых зависимостей — только `std` и `serde_json` для разбора весов.

use std::collections::HashMap;

/// Верхняя граница итераций (спека 29.1: «≤20 итераций»).
pub const MAX_ITER: usize = 20;
/// Минимальный damping (спека 29.1: d ∈ 0.5–0.85).
pub const DAMPING_MIN: f64 = 0.5;
/// Максимальный damping (стартовое значение на bench — 0.85).
pub const DAMPING_MAX: f64 = 0.85;
/// Дефолтная сходимость по L1-норме.
pub const DEFAULT_TOL: f64 = 1e-6;

/// Дефолтные веса рёбер по типу. Формат конфига —
/// JSON-объект `{"kind": weight}`; зафиксировано тестом `default_weights_are_fixed`.
pub const DEFAULT_PPR_WEIGHTS: &str =
    r#"{"manual":1.0,"contradicts":0.8,"causes":0.8,"category":0.5,"same_project":0.3}"#;

/// Вес ребра неизвестного типа.
pub const PPR_UNKNOWN_WEIGHT: f64 = 0.5;

/// Usage-Weighted PPR (столп III ob2h v2.0): верхняя граница access-boost,
/// `min(1 + ln(1 + access_count), CAP)`.
pub const PPR_ACCESS_CAP: f64 = 3.0;

/// Usage-Weighted PPR: grace-period — пока обращений меньше этого числа, узел
/// считается «нейтральным» (множитель `1.0`), чтобы свежие/редко читаемые записи
/// не искажали ранжирование.
pub const PPR_GRACE_ACCESSES: i64 = 4;

/// Usage-Weighted PPR / CATD-lite: за [`PPR_RECENCY_HALF_LIFE_SECS`] recency-множитель
/// `e^{-λ·Δt}` падает вдвое (≈ месяц — типовая давность знания в личном агенте).
pub const PPR_RECENCY_HALF_LIFE_SECS: f64 = 30.0 * 86_400.0;

/// Веса рёбер по типу: `kind → weight`.
pub type PprWeights = HashMap<String, f64>;

/// Ориентированное ребро с весом. Индексы — позиции узлов в [`PprGraph`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PprEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
}

/// Граф под PPR: `n` узлов, исходящие рёбра в CSR-подобном виде.
#[derive(Debug, Clone, Default)]
pub struct PprGraph {
    n: usize,
    /// `out[u]` — список `(сосед, вес)` исходящих рёбер узла `u`.
    out: Vec<Vec<(usize, f64)>>,
    /// Суммарный вес исходящих рёбер узла — знаменатель degree-normalization.
    out_weight: Vec<f64>,
}

impl PprGraph {
    /// Собрать граф из рёбер. Невалидные рёбра (индекс вне диапазона, не-конечный или
    /// неположительный вес) отбрасываются молча: вызывающий код подмешивает рёбра из
    /// разных источников, и одна битая строка не должна рушить поиск.
    pub fn new(n: usize, edges: &[PprEdge]) -> Self {
        let mut out: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        let mut out_weight = vec![0.0f64; n];
        for e in edges {
            if e.from >= n || e.to >= n || !e.weight.is_finite() || e.weight <= 0.0 {
                continue;
            }
            out[e.from].push((e.to, e.weight));
            out_weight[e.from] += e.weight;
        }
        Self { n, out, out_weight }
    }

    pub fn node_count(&self) -> usize {
        self.n
    }

    pub fn edge_count(&self) -> usize {
        self.out.iter().map(|v| v.len()).sum()
    }

    /// Суммарный вес исходящих рёбер узла (0.0 — висячий узел).
    pub fn out_weight(&self, node: usize) -> f64 {
        self.out_weight.get(node).copied().unwrap_or(0.0)
    }

    /// Исходящие рёбра узла: `(сосед, вес)`.
    pub fn neighbors(&self, node: usize) -> &[(usize, f64)] {
        self.out.get(node).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Статистика обращений к узлу для Usage-Weighted PPR (столп III ob2h v2.0).
///
/// Значения берутся из хранилища-источника: граф — `graph_nodes.access_count`/
/// `last_accessed`/`confidence`; память — `memories.access_count`/`last_accessed`/
/// `trust`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeUsage {
    /// Число реальных обращений к узлу.
    pub access_count: i64,
    /// Возраст последнего обращения в секундах (`None` — обращений не было).
    pub since_last_access_secs: Option<f64>,
    /// Доверие `0..=1` (память — `trust`; граф — `confidence`).
    pub trust: f64,
}

/// Множитель Usage-Weighted PPR (столп III ob2h v2.0):
/// `min(1 + ln(1 + access_count), CAP) · e^{-λ·Δt} · (0.1 + 0.9·trust)`, где
/// `λ = ln 2 / PPR_RECENCY_HALF_LIFE_SECS` (CATD-lite — экспоненциальный recency-decay).
///
/// Grace-period: пока `access_count < `[`PPR_GRACE_ACCESSES`], узел считается
/// нейтральным (`1.0`) — свежие и редко читаемые записи не искажают ранжирование.
pub fn usage_weight(u: NodeUsage) -> f64 {
    if u.access_count < PPR_GRACE_ACCESSES {
        return 1.0;
    }
    let accesses = u.access_count.max(0) as f64;
    let access_factor = (1.0 + (1.0 + accesses).ln()).min(PPR_ACCESS_CAP);
    let recency_factor = match u.since_last_access_secs {
        Some(dt) if dt.is_finite() && dt > 0.0 => {
            (-std::f64::consts::LN_2 * dt / PPR_RECENCY_HALF_LIFE_SECS).exp()
        }
        _ => 1.0,
    };
    let trust = u.trust.clamp(0.0, 1.0);
    access_factor * recency_factor * (0.1 + 0.9 * trust)
}

/// Персонализированный PageRank (без usage-весов). Возвращает `(узел, score)`,
/// отсортированные по убыванию score (узлы с нулевой массой не попадают в результат).
///
/// * `seeds` — персонализация: пары `(узел, вес)`, нормализуются в распределение.
/// * `damping` — клампится в [`DAMPING_MIN`]..=[`DAMPING_MAX`].
/// * `max_iter` — клампится в `1..=`[`MAX_ITER`].
/// * `tol` — порог L1-сходимости (`<= 0` → [`DEFAULT_TOL`]).
pub fn personalized_pagerank(
    graph: &PprGraph,
    seeds: &[(usize, f64)],
    damping: f64,
    max_iter: usize,
    tol: f64,
) -> Vec<(usize, f64)> {
    personalized_pagerank_weighted(graph, seeds, &[], damping, max_iter, tol)
}

/// Usage-Weighted Personalized PageRank (столп III ob2h v2.0).
///
/// `node_weights[v]` — множитель входящих в узел `v` рёбер (обычно [`usage_weight`]);
/// пустой срез — эквивалент [`personalized_pagerank`]. Эффективный вес ребра —
/// `w · node_weights[to]`, degree-normalization считается по взвешенной сумме
/// исходящих рёбер: часто читаемые узлы получают больше массы, но hub-защита не
/// даёт выдача «залипнуть». Невалидные множители (не-конечные/неположительные) → `1.0`.
pub fn personalized_pagerank_weighted(
    graph: &PprGraph,
    seeds: &[(usize, f64)],
    node_weights: &[f64],
    damping: f64,
    max_iter: usize,
    tol: f64,
) -> Vec<(usize, f64)> {
    let n = graph.n;
    if n == 0 || seeds.is_empty() {
        return Vec::new();
    }

    // Персонализация: суммируем веса сидов одного узла и нормализуем.
    let mut seed = vec![0.0f64; n];
    let mut total = 0.0f64;
    for &(node, w) in seeds {
        if node < n && w.is_finite() && w > 0.0 {
            seed[node] += w;
            total += w;
        }
    }
    if total <= 0.0 {
        return Vec::new();
    }
    for s in seed.iter_mut() {
        *s /= total;
    }

    // Usage-множитель узла (1.0 — если не задан или невалиден).
    let node_factor = |v: usize| -> f64 {
        node_weights
            .get(v)
            .copied()
            .filter(|f| f.is_finite() && *f > 0.0)
            .unwrap_or(1.0)
    };
    // Взвешенная сумма исходящих рёбер — знаменатель degree-normalization.
    // При пустых `node_weights` совпадает с `PprGraph::out_weight`.
    let mut wdeg = vec![0.0f64; n];
    for (u, deg) in wdeg.iter_mut().enumerate() {
        *deg = graph.out[u].iter().map(|&(v, w)| w * node_factor(v)).sum();
    }

    let d = damping.clamp(DAMPING_MIN, DAMPING_MAX);
    let iters = max_iter.clamp(1, MAX_ITER);
    let tol = if tol > 0.0 { tol } else { DEFAULT_TOL };

    let mut p = seed.clone();
    for _ in 0..iters {
        let mut next = vec![0.0f64; n];
        let mut dangling = 0.0f64;
        for (u, &pu) in p.iter().enumerate() {
            if pu == 0.0 {
                continue;
            }
            let deg = wdeg[u];
            if deg <= 0.0 {
                // Висячий узел: масса уходит в персонализацию (как в стандартном PPR).
                dangling += pu;
                continue;
            }
            // degree-normalization: hub со 100+ рёбрами отдаёт каждому соседу долю,
            // а не полную массу — иначе выдача «залипает» на хабе.
            for &(v, w) in &graph.out[u] {
                next[v] += d * pu * (w * node_factor(v) / deg);
            }
        }
        let teleport = (1.0 - d) + d * dangling;
        for (u, s) in seed.iter().enumerate() {
            if *s > 0.0 {
                next[u] += teleport * s;
            }
        }

        let l1: f64 = p.iter().zip(next.iter()).map(|(a, b)| (a - b).abs()).sum();
        p = next;
        if l1 < tol {
            break;
        }
    }

    let mut ranked: Vec<(usize, f64)> = p
        .into_iter()
        .enumerate()
        .filter(|(_, s)| *s > 0.0)
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

/// Разобрать веса рёбер из JSON-объекта `{"kind": weight}`.
/// Битые пары и неположительные веса игнорируются; пустой/невалидный JSON →
/// [`DEFAULT_PPR_WEIGHTS`] — конфиг не должен ломать поиск.
pub fn parse_ppr_weights(json: &str) -> PprWeights {
    let mut out = HashMap::new();
    let parsed: Option<HashMap<String, serde_json::Value>> = serde_json::from_str(json).ok();
    if let Some(map) = parsed {
        for (kind, val) in map {
            if let Some(w) = val.as_f64() {
                if w.is_finite() && w > 0.0 {
                    out.insert(kind.to_lowercase(), w);
                }
            }
        }
    }
    if out.is_empty() {
        return parse_ppr_weights(DEFAULT_PPR_WEIGHTS);
    }
    out
}

/// Вес ребра по типу: неизвестный kind → [`PPR_UNKNOWN_WEIGHT`].
pub fn weight_for(weights: &PprWeights, kind: &str) -> f64 {
    weights
        .get(&kind.to_lowercase())
        .copied()
        .unwrap_or(PPR_UNKNOWN_WEIGHT)
}

/// Нормализация сущности при построении рёбер: lowercase, `ё`→`е`,
/// выравнивание пунктуации, отбрасывание инициалов (однобуквенных токенов) и
/// схлопывание пробелов. Цель — меньше шумовых связей:
/// «И. Пресняков», «И.Пресняков» и «пресняков» дают одну метку.
pub fn normalize_entity(s: &str) -> String {
    let lowered = s.to_lowercase().replace('ё', "е");
    let mut cleaned = String::with_capacity(lowered.len());
    for ch in lowered.chars() {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            cleaned.push(ch);
        } else {
            cleaned.push(' ');
        }
    }
    cleaned
        .split_whitespace()
        // однобуквенные токены — инициалы/мусор; дефисные клеим в один токен
        .filter(|t| t.chars().filter(|c| c.is_alphanumeric()).count() > 1)
        .map(|t| {
            t.chars()
                .filter(|c| *c != '-' && *c != '_')
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(from: usize, to: usize, weight: f64) -> PprEdge {
        PprEdge { from, to, weight }
    }

    #[test]
    fn chain_reaches_multi_hop() {
        // 0=A, 1=B, 2=C
        let g = PprGraph::new(3, &[edge(0, 1, 1.0), edge(1, 2, 1.0)]);
        let ranked = personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        let score = |n: usize| {
            ranked
                .iter()
                .find(|(i, _)| *i == n)
                .map(|(_, s)| *s)
                .unwrap_or(0.0)
        };
        assert!(score(2) > 0.0, "C недостижим по цепочке A→B→C: {ranked:?}");
        assert!(
            score(1) > score(2),
            "ближний узел должен быть тяжелее дальнего"
        );
        assert!(score(0) > score(1), "seed — самый тяжёлый узел");
    }

    #[test]
    fn hub_does_not_flood_ranking() {
        // 0=seed S, 1=прямой сосед A, 2=hub H, 3..=листья H
        let mut edges = vec![edge(0, 1, 1.0), edge(0, 2, 1.0)];
        const LEAVES: usize = 120;
        for i in 0..LEAVES {
            edges.push(edge(2, 3 + i, 1.0));
        }
        let g = PprGraph::new(3 + LEAVES, &edges);
        let ranked = personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        let score = |n: usize| {
            ranked
                .iter()
                .find(|(i, _)| *i == n)
                .map(|(_, s)| *s)
                .unwrap_or(0.0)
        };

        let direct = score(1);
        let hub = score(2);
        let worst_leaf = (3..3 + LEAVES).map(score).fold(0.0f64, f64::max);
        assert!(direct > 0.0 && hub > 0.0);
        assert!(
            direct > worst_leaf,
            "прямой сосед обязан быть выше любого листа хаба (direct={direct}, leaf={worst_leaf})"
        );
        let leaf_mass: f64 = (3..3 + LEAVES).map(score).sum();
        assert!(
            leaf_mass < 2.0 * hub,
            "суммарная масса листьев не должна превышать массу хаба более чем вдвое"
        );
    }

    #[test]
    fn damping_and_iterations_are_clamped() {
        let g = PprGraph::new(2, &[edge(0, 1, 1.0)]);
        let low = personalized_pagerank(&g, &[(0, 1.0)], 0.0, 0, DEFAULT_TOL);
        let high = personalized_pagerank(&g, &[(0, 1.0)], 5.0, 999, DEFAULT_TOL);
        assert!(!low.is_empty() && !high.is_empty());
        let low_max = low.iter().map(|(_, s)| *s).fold(0.0f64, f64::max);
        let high_max = high.iter().map(|(_, s)| *s).fold(0.0f64, f64::max);
        assert!(
            (high_max - low_max).abs() > 1e-6,
            "damping обязан влиять на распределение"
        );
    }

    #[test]
    fn symmetric_seeds_give_equal_scores() {
        let g = PprGraph::new(2, &[edge(0, 1, 1.0), edge(1, 0, 1.0)]);
        let ranked = personalized_pagerank(&g, &[(0, 1.0), (1, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        assert_eq!(ranked.len(), 2);
        assert!((ranked[0].1 - ranked[1].1).abs() < 1e-9, "{ranked:?}");
    }

    #[test]
    fn mass_is_conserved_with_dangling_node() {
        let g = PprGraph::new(3, &[edge(0, 1, 1.0)]);
        let ranked = personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        let sum: f64 = ranked.iter().map(|(_, s)| *s).sum();
        assert!((sum - 1.0).abs() < 1e-3, "сумма масс = {sum}");
    }

    #[test]
    fn empty_inputs_are_safe() {
        let g = PprGraph::new(0, &[]);
        assert!(personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL).is_empty());
        let g2 = PprGraph::new(2, &[edge(0, 1, 1.0)]);
        assert!(personalized_pagerank(&g2, &[], 0.85, MAX_ITER, DEFAULT_TOL).is_empty());
        assert!(
            personalized_pagerank(&g2, &[(0, 0.0), (1, -1.0)], 0.85, MAX_ITER, DEFAULT_TOL)
                .is_empty()
        );
    }

    #[test]
    fn invalid_edges_are_skipped() {
        let g = PprGraph::new(
            2,
            &[
                edge(0, 1, 0.0),
                edge(0, 5, 1.0),
                edge(1, 0, f64::NAN),
                edge(0, 1, 0.7),
            ],
        );
        assert_eq!(g.edge_count(), 1);
        assert!((g.out_weight(0) - 0.7).abs() < 1e-12);
    }

    #[test]
    fn default_weights_are_fixed() {
        let w = parse_ppr_weights(DEFAULT_PPR_WEIGHTS);
        assert_eq!(w.len(), 5);
        assert!((w["manual"] - 1.0).abs() < 1e-12);
        assert!((w["contradicts"] - 0.8).abs() < 1e-12);
        assert!((w["causes"] - 0.8).abs() < 1e-12);
        assert!((w["category"] - 0.5).abs() < 1e-12);
        assert!((w["same_project"] - 0.3).abs() < 1e-12);
        assert!((weight_for(&w, "manual") - 1.0).abs() < 1e-12);
        assert!((weight_for(&w, "unknown-kind") - PPR_UNKNOWN_WEIGHT).abs() < 1e-12);
        assert_eq!(parse_ppr_weights("не json").len(), 5);
        assert_eq!(parse_ppr_weights("").len(), 5);
    }

    #[test]
    fn entity_normalization_collapses_forms() {
        assert_eq!(
            normalize_entity("И. Пресняков"),
            normalize_entity("ПРЕСНЯКОВ")
        );
        assert_eq!(
            normalize_entity("И.Пресняков"),
            normalize_entity("и пресняков")
        );
        assert_eq!(normalize_entity("Ёлка"), "елка");
        assert_eq!(normalize_entity("ООО «Ромашка»"), "ооо ромашка");
        assert_eq!(normalize_entity("кое-что"), "коечто");
    }

    fn usage(access_count: i64, since: Option<f64>, trust: f64) -> NodeUsage {
        NodeUsage {
            access_count,
            since_last_access_secs: since,
            trust,
        }
    }

    #[test]
    fn usage_weight_applies_grace_period() {
        // До grace-порога множитель нейтрален, даже при высоком trust.
        assert!((usage_weight(usage(0, None, 1.0)) - 1.0).abs() < 1e-12);
        assert!((usage_weight(usage(3, Some(1.0), 1.0)) - 1.0).abs() < 1e-12);
        // На пороге и выше — буст включается.
        assert!(usage_weight(usage(PPR_GRACE_ACCESSES, None, 1.0)) > 1.0);
    }

    #[test]
    fn usage_weight_grows_with_access_and_caps() {
        let at_grace = usage_weight(usage(4, None, 1.0));
        let expected = 1.0 + 5.0f64.ln();
        assert!((at_grace - expected).abs() < 1e-9, "got {at_grace}");
        // Дальше растёт, но не превышает cap.
        assert!(usage_weight(usage(10, None, 1.0)) > at_grace);
        assert!((usage_weight(usage(1000, None, 1.0)) - PPR_ACCESS_CAP).abs() < 1e-12);
    }

    #[test]
    fn usage_weight_decays_with_recency() {
        let fresh = usage_weight(usage(10, Some(0.0), 1.0));
        let half = usage_weight(usage(10, Some(PPR_RECENCY_HALF_LIFE_SECS), 1.0));
        let late = usage_weight(usage(10, Some(2.0 * PPR_RECENCY_HALF_LIFE_SECS), 1.0));
        assert!(fresh > half && half > late, "{fresh} {half} {late}");
        assert!((half / fresh - 0.5).abs() < 1e-9, "half-life должен давать 1/2");
        // Отсутствие данных о давности — нейтрально (без штрафа).
        assert!((usage_weight(usage(10, None, 1.0)) - fresh).abs() < 1e-12);
    }

    #[test]
    fn usage_weight_scales_with_trust() {
        let high = usage_weight(usage(10, None, 1.0));
        let mid = usage_weight(usage(10, None, 0.5));
        let low = usage_weight(usage(10, None, 0.0));
        assert!(high > mid && mid > low, "{high} {mid} {low}");
        // trust=1.0 → множитель 1.0, trust=0.0 → 0.1.
        assert!((low / high - 0.1).abs() < 1e-9);
        // trust вне [0,1] клампится, а не искажает.
        assert!((usage_weight(usage(10, None, 5.0)) - high).abs() < 1e-12);
    }

    #[test]
    fn weighted_pagerank_with_empty_weights_matches_base() {
        let g = PprGraph::new(3, &[edge(0, 1, 1.0), edge(1, 2, 1.0)]);
        let base = personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        let weighted =
            personalized_pagerank_weighted(&g, &[(0, 1.0)], &[], 0.85, MAX_ITER, DEFAULT_TOL);
        assert_eq!(base, weighted);
    }

    #[test]
    fn weighted_pagerank_boosts_hot_target() {
        // 0=seed, 1=cold A, 2=hot B (равные базовые веса рёбер).
        let g = PprGraph::new(3, &[edge(0, 1, 1.0), edge(0, 2, 1.0)]);
        let score = |r: &[(usize, f64)], n: usize| {
            r.iter()
                .find(|(i, _)| *i == n)
                .map(|(_, s)| *s)
                .unwrap_or(0.0)
        };

        let base = personalized_pagerank(&g, &[(0, 1.0)], 0.85, MAX_ITER, DEFAULT_TOL);
        assert!(
            (score(&base, 1) - score(&base, 2)).abs() < 1e-9,
            "без usage-весов симметричные узлы равны"
        );

        let weighted = personalized_pagerank_weighted(
            &g,
            &[(0, 1.0)],
            &[1.0, 1.0, 3.0],
            0.85,
            MAX_ITER,
            DEFAULT_TOL,
        );
        assert!(
            score(&weighted, 2) > score(&weighted, 1),
            "hot-узел B обязан обойти cold-узел A: {weighted:?}"
        );
    }
}
