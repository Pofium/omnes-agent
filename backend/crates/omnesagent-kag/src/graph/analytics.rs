use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodNodeInfo {
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub file_path: Option<String>,
    pub in_degree: usize,
    pub out_degree: usize,
    pub total_degree: usize,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedNode {
    pub id: i64,
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub file_path: Option<String>,
    pub depth: usize,
    pub edge_label: String,
    pub is_god_node: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "🟢 LOW"),
            RiskLevel::Medium => write!(f, "🟡 MEDIUM"),
            RiskLevel::High => write!(f, "🔴 HIGH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target_symbol: String,
    pub target_file: Option<String>,
    pub target_type: Option<String>,
    pub max_depth: usize,
    pub affected_nodes: Vec<AffectedNode>,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<String>,
    pub markdown_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub nodes: Vec<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub component: String,
    pub afferent_ca: usize,
    pub efferent_ce: usize,
    pub instability: f32,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReport {
    pub project_id: String,
    pub project_name: String,
    pub root_path: String,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub god_nodes: Vec<GodNodeInfo>,
    pub node_types_count: HashMap<String, usize>,
    pub top_dependencies: Vec<(String, usize)>,
    pub circular_dependencies: Vec<CircularDependency>,
    pub component_metrics: Vec<ComponentMetrics>,
    pub markdown_summary: String,
}

pub struct GraphAnalytics;

impl GraphAnalytics {
    /// Вычисляет степень связности узлов и маркирует «узлы-боги» (God Nodes) в SQLite.
    pub fn update_god_nodes(conn: &Connection, project_id: &str) -> Result<Vec<GodNodeInfo>> {
        // 1. Сбрасываем старые god nodes для проекта
        conn.execute(
            "UPDATE graph_nodes SET is_god_node = 0 WHERE project_id = ?1",
            params![project_id],
        )?;

        // 2. Считаем in-degree и out-degree для каждого узла
        let mut degree_map: HashMap<i64, (usize, usize)> = HashMap::new(); // pk -> (in_degree, out_degree)

        let mut stmt = conn.prepare(
            r#"
            SELECT source_id, target_id 
            FROM graph_edges 
            WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')
            "#,
        )?;

        let rows = stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;

        for edge in rows {
            let (src, dst) = edge?;
            degree_map.entry(src).or_insert((0, 0)).1 += 1;
            degree_map.entry(dst).or_insert((0, 0)).0 += 1;
        }

        if degree_map.is_empty() {
            return Ok(Vec::new());
        }

        // Собираем узлы с их общим degree
        let mut node_scores: Vec<(i64, usize, usize, usize)> = degree_map
            .into_iter()
            .map(|(pk, (in_deg, out_deg))| (pk, in_deg, out_deg, in_deg + out_deg))
            .collect();

        node_scores.sort_by_key(|b| std::cmp::Reverse(b.3));

        // Берем топ 10% или минимум топ-5 наиболее связанных узлов
        let top_count = (node_scores.len() / 10).clamp(3, 20).min(node_scores.len());
        let god_nodes_pks: Vec<i64> = node_scores
            .iter()
            .take(top_count)
            .map(|(pk, ..)| *pk)
            .collect();

        // Маркируем в базе
        for pk in &god_nodes_pks {
            conn.execute(
                "UPDATE graph_nodes SET is_god_node = 1 WHERE id = ?1",
                params![pk],
            )?;
        }

        // Загружаем подробную информацию о God Nodes
        let mut god_nodes_info = Vec::new();
        for (pk, in_deg, out_deg, total) in node_scores.into_iter().take(top_count) {
            let mut q_stmt = conn.prepare(
                "SELECT node_id, label, node_type, file_path, description FROM graph_nodes WHERE id = ?1",
            )?;
            if let Ok(row) = q_stmt.query_row(params![pk], |r| {
                Ok(GodNodeInfo {
                    node_id: r.get(0)?,
                    label: r.get(1)?,
                    node_type: r.get(2)?,
                    file_path: r.get(3)?,
                    in_degree: in_deg,
                    out_degree: out_deg,
                    total_degree: total,
                    description: r.get(4)?,
                })
            }) {
                god_nodes_info.push(row);
            }
        }

        Ok(god_nodes_info)
    }

    /// Генерирует исчерпывающий архитектурный дайджест проекта.
    pub fn generate_project_report(conn: &Connection, project_id: &str) -> Result<ProjectReport> {
        let (name, root_path, tech_stack): (String, String, Option<String>) = conn.query_row(
            "SELECT name, root_path, tech_stack FROM projects WHERE id = ?1",
            params![project_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;

        let god_nodes = Self::update_god_nodes(conn, project_id)?;

        let total_nodes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        let total_edges: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')",
            params![project_id],
            |r| r.get(0),
        ).unwrap_or(0);

        // Распределение по типам узлов
        let mut node_types_count = HashMap::new();
        let mut type_stmt = conn.prepare(
            r#"
            SELECT node_type, COUNT(*) 
            FROM graph_nodes 
            WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')
            GROUP BY node_type
            "#,
        )?;
        let type_rows = type_stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
        })?;
        for tr in type_rows {
            let (ntype, count) = tr?;
            node_types_count.insert(ntype, count);
        }

        // Топ внешних зависимостей / модулей
        let mut top_deps = Vec::new();
        let mut dep_stmt = conn.prepare(
            r#"
            SELECT target_id, COUNT(*) as cnt
            FROM graph_edges
            WHERE project_id = ?1 AND label = 'IMPORTS' AND (deleted_at IS NULL OR deleted_at = '')
            GROUP BY target_id
            ORDER BY cnt DESC
            LIMIT 10
            "#,
        )?;
        let dep_rows = dep_stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? as usize))
        })?;
        for dr in dep_rows {
            let (target_pk, cnt) = dr?;
            if let Ok(label) = conn.query_row(
                "SELECT label FROM graph_nodes WHERE id = ?1",
                params![target_pk],
                |r| r.get::<_, String>(0),
            ) {
                top_deps.push((label, cnt));
            }
        }

        let circular_dependencies = Self::find_circular_dependencies(conn, project_id)?;
        let component_metrics = Self::compute_coupling_metrics(conn, project_id)?;

        // Формируем Markdown дайджест
        let mut md = String::new();
        md.push_str(&format!(
            "# 🏛️ Архитектурный дайджест проекта: {}\n\n",
            name
        ));
        md.push_str(&format!("- **ID:** `{}`\n", project_id));
        md.push_str(&format!("- **Корневой путь:** `{}`\n", root_path));
        if let Some(ts) = tech_stack {
            md.push_str(&format!("- **Стек технологий:** {}\n", ts));
        }
        md.push_str(&format!(
            "- **Узлов в графе:** {} | **Связей:** {}\n\n",
            total_nodes, total_edges
        ));

        md.push_str("## 👑 Ключевые архитектурные хабы (God Nodes)\n");
        md.push_str("Центральные структуры, модули и сервисы с максимальной связностью:\n\n");
        for gn in &god_nodes {
            let file_str = gn.file_path.as_deref().unwrap_or("внешний");
            md.push_str(&format!(
                "- **`{}`** (`{}`) — связей: {} (in: {}, out: {})\n  ↳ Файл: `{}`\n",
                gn.label, gn.node_type, gn.total_degree, gn.in_degree, gn.out_degree, file_str
            ));
        }

        md.push_str("\n## 📊 Компоненты кодовой базы\n");
        for (ntype, count) in &node_types_count {
            md.push_str(&format!("- **{}:** {}\n", ntype, count));
        }

        if !top_deps.is_empty() {
            md.push_str("\n## 📦 Наиболее используемые зависимости и модули\n");
            for (dep, cnt) in &top_deps {
                md.push_str(&format!("- `{}` (используется в {} местах)\n", dep, cnt));
            }
        }

        md.push_str("\n## 🔄 Циклические зависимости\n");
        if circular_dependencies.is_empty() {
            md.push_str("✅ Циклических зависимостей не обнаружено (архитектура ациклична).\n");
        } else {
            md.push_str("⚠️ **Обнаружены потенциально проблемные циклы:**\n");
            for (i, cd) in circular_dependencies.iter().take(5).enumerate() {
                md.push_str(&format!(
                    "{}. Длина {}: {}\n",
                    i + 1,
                    cd.length,
                    cd.nodes.join(" ➔ ")
                ));
            }
        }

        if !component_metrics.is_empty() {
            md.push_str("\n## 📐 Архитектурная стабильность компонентов (Роберт Мартин)\n");
            let top_stable: Vec<&ComponentMetrics> = component_metrics.iter().take(4).collect();
            let top_unstable: Vec<&ComponentMetrics> =
                component_metrics.iter().rev().take(4).collect();

            md.push_str("**Наиболее стабильные компоненты (Ядро, малый Instability I):**\n");
            for c in top_stable {
                md.push_str(&format!(
                    "- `{}`: I={:.2} (Ca={}, Ce={}) [{}]\n",
                    c.component, c.instability, c.afferent_ca, c.efferent_ce, c.category
                ));
            }
            md.push_str(
                "\n**Наиболее гибкие/нестабильные компоненты (Листья, высокий Instability I):**\n",
            );
            for c in top_unstable {
                md.push_str(&format!(
                    "- `{}`: I={:.2} (Ca={}, Ce={}) [{}]\n",
                    c.component, c.instability, c.afferent_ca, c.efferent_ce, c.category
                ));
            }
        }

        // Кандидаты в мертвый код
        if let Ok(dead) = crate::graph::callpath::dead_code(conn, project_id) {
            if !dead.is_empty() {
                md.push_str("\n## 🧟 Кандидаты в мёртвый код (in-degree 0)\n");
                md.push_str(&crate::graph::callpath::format_dead_code(&dead, 8));
            }
        }

        // Архитектурные зоны (Communities)
        if let Ok(zones_report) = crate::graph::communities::detect_zones(conn, project_id, 2) {
            if !zones_report.zones.is_empty() {
                md.push_str(&crate::graph::communities::format_zones(&zones_report));
            }
        }

        Ok(ProjectReport {
            project_id: project_id.to_string(),
            project_name: name,
            root_path,
            total_nodes: total_nodes as usize,
            total_edges: total_edges as usize,
            god_nodes,
            node_types_count,
            top_dependencies: top_deps,
            circular_dependencies,
            component_metrics,
            markdown_summary: md,
        })
    }

    /// Анализ радиуса изменений (Blast Radius) для функции, класса или файла.
    pub fn analyze_impact(
        conn: &Connection,
        project_id: &str,
        symbol_or_path: &str,
        max_depth: usize,
    ) -> Result<ImpactReport> {
        let max_depth = max_depth.clamp(1, 10);

        // 1. Поиск целевого узла
        let mut target_stmt = conn.prepare(
            r#"
            SELECT id, node_id, label, node_type, file_path, is_god_node
            FROM graph_nodes
            WHERE project_id = ?1
              AND (label = ?2 OR label LIKE ?3 OR file_path = ?2 OR file_path LIKE ?3)
              AND (deleted_at IS NULL OR deleted_at = '')
            ORDER BY 
              CASE 
                WHEN label = ?2 THEN 0
                WHEN file_path = ?2 THEN 1
                ELSE 2
              END
            LIMIT 1
            "#,
        )?;

        let pattern = format!("%{}%", symbol_or_path);
        let target_node =
            target_stmt.query_row(params![project_id, symbol_or_path, pattern], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, i64>(5).unwrap_or(0) == 1,
                ))
            });

        let (target_pk, _target_nid, target_label, target_type, target_file, target_is_god) =
            match target_node {
                Ok(n) => n,
                Err(_) => {
                    return Ok(ImpactReport {
                        target_symbol: symbol_or_path.to_string(),
                        target_file: None,
                        target_type: None,
                        max_depth,
                        affected_nodes: Vec::new(),
                        risk_level: RiskLevel::Low,
                        risk_factors: vec!["Символ не найден в графе проекта.".to_string()],
                        markdown_summary: format!(
                            "# 💥 Анализ радиуса изменений (Blast Radius)\n\nСимвол или файл `{}` не найден в графе проекта `{}`.",
                            symbol_or_path, project_id
                        ),
                    });
                }
            };

        // 2. BFS обход обратных связей (кто зависит от target)
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(target_pk);

        let mut queue: VecDeque<(i64, usize)> = VecDeque::new();
        queue.push_back((target_pk, 0));

        let mut affected_nodes: Vec<AffectedNode> = Vec::new();

        let mut in_edge_stmt = conn.prepare(
            r#"
            SELECT e.source_id, e.label, n.node_id, n.label, n.node_type, n.file_path, n.is_god_node
            FROM graph_edges e
            JOIN graph_nodes n ON n.id = e.source_id
            WHERE e.target_id = ?1
              AND (e.deleted_at IS NULL OR e.deleted_at = '')
              AND (n.deleted_at IS NULL OR n.deleted_at = '')
            "#,
        )?;

        while let Some((current_pk, current_depth)) = queue.pop_front() {
            if current_depth >= max_depth {
                continue;
            }

            let rows = in_edge_stmt.query_map(params![current_pk], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, i64>(6).unwrap_or(0) == 1,
                ))
            })?;

            for r in rows {
                let (src_pk, edge_lbl, n_id, n_lbl, n_type, f_path, is_god) = r?;
                if !visited.contains(&src_pk) {
                    visited.insert(src_pk);
                    let next_depth = current_depth + 1;
                    affected_nodes.push(AffectedNode {
                        id: src_pk,
                        node_id: n_id,
                        label: n_lbl,
                        node_type: n_type,
                        file_path: f_path,
                        depth: next_depth,
                        edge_label: edge_lbl,
                        is_god_node: is_god,
                    });
                    queue.push_back((src_pk, next_depth));
                }
            }
        }

        // 3. Вычисление факторов риска
        let mut risk_factors = Vec::new();
        if target_is_god {
            risk_factors
                .push("Целевой узел является ключевым архитектурным хабом (God Node)".to_string());
        }

        let god_nodes_affected: Vec<&AffectedNode> =
            affected_nodes.iter().filter(|n| n.is_god_node).collect();
        for gn in &god_nodes_affected {
            risk_factors.push(format!(
                "Затрагивает архитектурный хаб `{}` ({}) на глубине {}",
                gn.label, gn.node_type, gn.depth
            ));
        }

        if affected_nodes.len() > 10 {
            risk_factors.push(format!(
                "Критически широкий охват зависимостей: {} узлов",
                affected_nodes.len()
            ));
        } else if affected_nodes.len() > 4 {
            risk_factors.push(format!(
                "Умеренный радиус влияния: {} зависимых узлов",
                affected_nodes.len()
            ));
        }

        let risk_level =
            if target_is_god || !god_nodes_affected.is_empty() || affected_nodes.len() > 8 {
                RiskLevel::High
            } else if affected_nodes.len() > 2 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

        // 4. Построение Markdown-отчета
        let mut md = String::new();
        md.push_str("# 💥 Анализ радиуса изменений (Blast Radius)\n\n");
        md.push_str(&format!(
            "- **Целевой символ:** `{}` [{}]\n",
            target_label, target_type
        ));
        if let Some(ref f) = target_file {
            md.push_str(&format!("- **Файл:** `{}`\n", f));
        }
        md.push_str(&format!("- **Уровень риска:** {}\n", risk_level));
        md.push_str(&format!(
            "- **Затронутых зависимых узлов:** {}\n",
            affected_nodes.len()
        ));
        md.push_str(&format!("- **Глубина обхода графа:** {}\n\n", max_depth));

        if !risk_factors.is_empty() {
            md.push_str("### ⚠️ Факторы риска:\n");
            for rf in &risk_factors {
                md.push_str(&format!("- {}\n", rf));
            }
            md.push('\n');
        }

        if affected_nodes.is_empty() {
            md.push_str("✅ Зависимых компонентов не обнаружено. Символ изолирован или является конечным потребителем.\n");
        } else {
            md.push_str("### 🌳 Дерево затронутых компонентов:\n\n");
            let mut by_depth: HashMap<usize, Vec<&AffectedNode>> = HashMap::new();
            for n in &affected_nodes {
                by_depth.entry(n.depth).or_default().push(n);
            }
            for d in 1..=max_depth {
                if let Some(nodes) = by_depth.get(&d) {
                    md.push_str(&format!(
                        "**Уровень {} ({}):**\n",
                        d,
                        if d == 1 {
                            "Прямые потребители"
                        } else {
                            "Косвенное влияние"
                        }
                    ));
                    for n in nodes {
                        let loc = n.file_path.as_deref().unwrap_or("-");
                        let god_mark = if n.is_god_node { " [👑 GodNode]" } else { "" };
                        md.push_str(&format!(
                            "- `{}` [{}] ({}) через `{}`{}\n",
                            n.label, n.node_type, loc, n.edge_label, god_mark
                        ));
                    }
                    md.push('\n');
                }
            }

            md.push_str("### 🛡️ Рекомендации перед изменением:\n");
            match risk_level {
                RiskLevel::High => {
                    md.push_str("- ❗ Обязательно согласуйте интерфейс перед рефакторингом.\n");
                    md.push_str("- Запустите полный тестовый сьют затронутых модулей.\n");
                    md.push_str("- Не меняйте публичные сигнатуры без обратной совместимости.\n");
                }
                RiskLevel::Medium => {
                    md.push_str("- Проверьте вызовы прямых потребителей (Уровень 1).\n");
                    md.push_str("- Напишите или обновите модульные тесты для зависимых функций.\n");
                }
                RiskLevel::Low => {
                    md.push_str("- Локальное изменение с минимальным риском регрессии.\n");
                }
            }
        }

        Ok(ImpactReport {
            target_symbol: target_label,
            target_file,
            target_type: Some(target_type),
            max_depth,
            affected_nodes,
            risk_level,
            risk_factors,
            markdown_summary: md,
        })
    }

    /// Детектор циклических зависимостей алгоритмом Тарьяна (Tarjan's SCC).
    pub fn find_circular_dependencies(
        conn: &Connection,
        project_id: &str,
    ) -> Result<Vec<CircularDependency>> {
        let mut node_stmt = conn.prepare(
            r#"
            SELECT id, label, file_path
            FROM graph_nodes
            WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')
            "#,
        )?;
        let node_rows = node_stmt.query_map(params![project_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut node_labels: HashMap<i64, String> = HashMap::new();
        for nr in node_rows {
            let (pk, lbl, fpath) = nr?;
            let display = if let Some(f) = fpath {
                format!("{} ({})", lbl, f)
            } else {
                lbl
            };
            node_labels.insert(pk, display);
        }

        let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut edge_stmt = conn.prepare(
            r#"
            SELECT source_id, target_id
            FROM graph_edges
            WHERE project_id = ?1 AND (deleted_at IS NULL OR deleted_at = '')
            "#,
        )?;
        let edge_rows = edge_stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
        for er in edge_rows {
            let (src, dst) = er?;
            adj.entry(src).or_default().push(dst);
        }

        struct TarjanContext<'a> {
            adj: &'a HashMap<i64, Vec<i64>>,
            indices: HashMap<i64, usize>,
            lowlinks: HashMap<i64, usize>,
            on_stack: HashSet<i64>,
            stack: Vec<i64>,
            index: usize,
            sccs: Vec<Vec<i64>>,
        }

        impl<'a> TarjanContext<'a> {
            fn strongconnect(&mut self, v: i64) {
                self.indices.insert(v, self.index);
                self.lowlinks.insert(v, self.index);
                self.index += 1;
                self.stack.push(v);
                self.on_stack.insert(v);

                if let Some(neighbors) = self.adj.get(&v) {
                    for &w in neighbors {
                        if !self.indices.contains_key(&w) {
                            self.strongconnect(w);
                            let v_low = self.lowlinks[&v];
                            let w_low = self.lowlinks[&w];
                            self.lowlinks.insert(v, v_low.min(w_low));
                        } else if self.on_stack.contains(&w) {
                            let v_low = self.lowlinks[&v];
                            let w_idx = self.indices[&w];
                            self.lowlinks.insert(v, v_low.min(w_idx));
                        }
                    }
                }

                if self.lowlinks[&v] == self.indices[&v] {
                    let mut scc = Vec::new();
                    while let Some(w) = self.stack.pop() {
                        self.on_stack.remove(&w);
                        scc.push(w);
                        if w == v {
                            break;
                        }
                    }
                    let is_cycle = if scc.len() > 1 {
                        true
                    } else if scc.len() == 1 {
                        self.adj
                            .get(&scc[0])
                            .map(|nbrs| nbrs.contains(&scc[0]))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if is_cycle {
                        self.sccs.push(scc);
                    }
                }
            }
        }

        let mut ctx = TarjanContext {
            adj: &adj,
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            index: 0,
            sccs: Vec::new(),
        };

        for &node_id in node_labels.keys() {
            if !ctx.indices.contains_key(&node_id) {
                ctx.strongconnect(node_id);
            }
        }

        let mut circular_deps = Vec::new();
        for scc in ctx.sccs {
            let names: Vec<String> = scc
                .iter()
                .map(|id| {
                    node_labels
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| format!("node_{id}"))
                })
                .collect();
            let len = names.len();
            circular_deps.push(CircularDependency {
                nodes: names,
                length: len,
            });
        }

        circular_deps.sort_by_key(|b| std::cmp::Reverse(b.length));
        Ok(circular_deps)
    }

    /// Вычисляет метрики связанности и стабильности пакетов Роберта Мартина (Ca, Ce, I).
    pub fn compute_coupling_metrics(
        conn: &Connection,
        project_id: &str,
    ) -> Result<Vec<ComponentMetrics>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT src.file_path, dst.file_path
            FROM graph_edges e
            JOIN graph_nodes src ON src.id = e.source_id
            JOIN graph_nodes dst ON dst.id = e.target_id
            WHERE e.project_id = ?1
              AND src.file_path IS NOT NULL AND dst.file_path IS NOT NULL
              AND src.file_path != dst.file_path
              AND (e.deleted_at IS NULL OR e.deleted_at = '')
            "#,
        )?;

        let mut afferent: HashMap<String, HashSet<String>> = HashMap::new();
        let mut efferent: HashMap<String, HashSet<String>> = HashMap::new();
        let mut all_files: HashSet<String> = HashSet::new();

        let rows = stmt.query_map(params![project_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;

        for r in rows {
            let (src_file, dst_file) = r?;
            all_files.insert(src_file.clone());
            all_files.insert(dst_file.clone());

            efferent
                .entry(src_file.clone())
                .or_default()
                .insert(dst_file.clone());
            afferent.entry(dst_file).or_default().insert(src_file);
        }

        let mut metrics = Vec::new();
        for file in all_files {
            let ca = afferent.get(&file).map(|s| s.len()).unwrap_or(0);
            let ce = efferent.get(&file).map(|s| s.len()).unwrap_or(0);
            let total = ca + ce;
            let instability = if total == 0 {
                0.0
            } else {
                ce as f32 / total as f32
            };

            let category = if instability < 0.3 {
                "Стабильный (Ядро)".to_string()
            } else if instability <= 0.7 {
                "Сбалансированный".to_string()
            } else {
                "Нестабильный (Лист)".to_string()
            };

            metrics.push(ComponentMetrics {
                component: file,
                afferent_ca: ca,
                efferent_ce: ce,
                instability,
                category,
            });
        }

        metrics.sort_by(|a, b| {
            a.instability
                .partial_cmp(&b.instability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(metrics)
    }

    /// Сборка сжатого блока `<project_context>` для системного промпта агента.
    pub fn build_project_context(
        conn: &Connection,
        project_id: &str,
        task_query: Option<&str>,
    ) -> Result<String> {
        let report = Self::generate_project_report(conn, project_id)?;
        let mut ctx = String::new();
        ctx.push_str(&format!("<project_context id=\"{}\">\n", project_id));
        ctx.push_str(&format!(
            "Project: {} (Root: {})\n",
            report.project_name, report.root_path
        ));

        ctx.push_str("Core Architecture Hubs (God Nodes):\n");
        for gn in report.god_nodes.iter().take(8) {
            let loc = gn.file_path.as_deref().unwrap_or("");
            ctx.push_str(&format!(
                "- {} [{}] ({}) -> {} connections\n",
                gn.label, gn.node_type, loc, gn.total_degree
            ));
        }

        if let Some(query) = task_query {
            ctx.push_str(&format!("\nRelevant Subsystems for Task '{}':\n", query));
            let mut stmt = conn.prepare(
                r#"
                SELECT label, node_type, file_path, description 
                FROM graph_nodes 
                WHERE project_id = ?1 AND (label LIKE ?2 OR description LIKE ?2)
                LIMIT 5
                "#,
            )?;
            let query_like = format!("%{}%", query);
            let rows = stmt.query_map(params![project_id, query_like], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            })?;
            for row in rows {
                let (lbl, ntype, fpath, desc) = row?;
                ctx.push_str(&format!(
                    "- {} [{}] in {:?}: {:?}\n",
                    lbl, ntype, fpath, desc
                ));
            }
        }

        ctx.push_str("</project_context>");
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixed graph for project 'p1':
    ///   extra.rs ─IMPORTS→ hub.rs ←IMPORTS─ mid.rs ←IMPORTS─ leaf.rs
    ///   cyc_a.rs ⇄ cyc_b.rs (IMPORTS cycle), cyc_a.rs ─CALLS→ hub.rs
    ///   lonely.rs has no edges; deleted.rs is soft-deleted.
    /// Node ids are deterministic (1..8, AUTOINCREMENT order).
    fn setup(conn: &Connection) {
        conn.execute_batch(
            r#"
            INSERT INTO projects (id, name, root_path, tech_stack, created_at, updated_at)
            VALUES ('p1', 'demo', 'C:/demo', 'rust', 't', 't'),
                   ('p2', 'empty', 'C:/p2', NULL, 't', 't');

            INSERT INTO graph_nodes (id, node_id, label, node_type, file_path, description, project_id, created_at, updated_at) VALUES
              (1, 'n1', 'hub.rs',    'File', 'src/hub.rs',    'central hub', 'p1', 't', 't'),
              (2, 'n2', 'mid.rs',    'File', 'src/mid.rs',    NULL,          'p1', 't', 't'),
              (3, 'n3', 'leaf.rs',   'File', 'src/leaf.rs',   NULL,          'p1', 't', 't'),
              (4, 'n4', 'extra.rs',  'File', 'src/extra.rs',  NULL,          'p1', 't', 't'),
              (5, 'n5', 'cyc_a.rs',  'File', 'src/cyc_a.rs',  NULL,          'p1', 't', 't'),
              (6, 'n6', 'cyc_b.rs',  'File', 'src/cyc_b.rs',  NULL,          'p1', 't', 't'),
              (7, 'n7', 'deleted.rs','File', 'src/gone.rs',   NULL,          'p1', 't', 't'),
              (8, 'n8', 'lonely.rs', 'File', 'src/lonely.rs', NULL,          'p1', 't', 't');

            UPDATE graph_nodes SET deleted_at = '2026-01-01' WHERE id = 7;

            INSERT INTO graph_edges (source_id, target_id, label, project_id, created_at, updated_at) VALUES
              (2, 1, 'IMPORTS', 'p1', 't', 't'),
              (3, 2, 'IMPORTS', 'p1', 't', 't'),
              (4, 1, 'IMPORTS', 'p1', 't', 't'),
              (5, 6, 'IMPORTS', 'p1', 't', 't'),
              (6, 5, 'IMPORTS', 'p1', 't', 't'),
              (5, 1, 'CALLS',   'p1', 't', 't');
            "#,
        )
        .unwrap();
    }

    fn setup_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::migrate(&conn).unwrap();
        setup(&conn);
        conn
    }

    #[test]
    fn risk_level_display_uses_colored_labels() {
        assert_eq!(RiskLevel::Low.to_string(), "🟢 LOW");
        assert_eq!(RiskLevel::Medium.to_string(), "🟡 MEDIUM");
        assert_eq!(RiskLevel::High.to_string(), "🔴 HIGH");
    }

    #[test]
    fn update_god_nodes_marks_top_degree_and_resets_stale_flags() {
        let conn = setup_in_memory();
        // Stale flag that must be wiped by the reset pass.
        conn.execute("UPDATE graph_nodes SET is_god_node = 1 WHERE id = 8", [])
            .unwrap();

        let gods = GraphAnalytics::update_god_nodes(&conn, "p1").unwrap();

        // 6 nodes carry edges → top_count = clamp(6/10, 3, 20) = 3.
        assert_eq!(gods.len(), 3);
        // hub.rs (in 3) and cyc_a.rs (out 2 + in 1) always win the top.
        let hub = gods
            .iter()
            .find(|g| g.label == "hub.rs")
            .expect("hub is a god node");
        assert_eq!(hub.in_degree, 3);
        assert_eq!(hub.out_degree, 0);
        assert_eq!(hub.total_degree, 3);
        let cyc_a = gods
            .iter()
            .find(|g| g.label == "cyc_a.rs")
            .expect("cyc_a is a god node");
        assert_eq!(cyc_a.total_degree, 3);

        let flagged: Vec<i64> = conn
            .prepare("SELECT id FROM graph_nodes WHERE is_god_node = 1 AND project_id = 'p1'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(flagged.len(), 3);
        assert!(
            !flagged.contains(&8),
            "stale god flag on edge-less node is reset"
        );

        // Empty project has no edges at all → no god nodes.
        assert!(
            GraphAnalytics::update_god_nodes(&conn, "p2")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn project_report_aggregates_counts_deps_cycles_and_markdown() {
        let conn = setup_in_memory();
        let report = GraphAnalytics::generate_project_report(&conn, "p1").unwrap();

        assert_eq!(report.project_id, "p1");
        assert_eq!(report.project_name, "demo");
        assert_eq!(report.root_path, "C:/demo");
        assert_eq!(report.total_nodes, 7, "soft-deleted node is excluded");
        assert_eq!(report.total_edges, 6);
        assert_eq!(report.node_types_count.get("File"), Some(&7));

        let (top_label, top_count) = report
            .top_dependencies
            .first()
            .expect("hub has IMPORTS fans");
        assert_eq!(top_label, "hub.rs");
        assert_eq!(*top_count, 2, "mid.rs and extra.rs import hub.rs");

        assert_eq!(report.circular_dependencies.len(), 1);
        assert_eq!(report.circular_dependencies[0].length, 2);

        assert!(report.god_nodes.iter().any(|g| g.label == "hub.rs"));
        assert!(
            report
                .markdown_summary
                .contains("Архитектурный дайджест проекта: demo")
        );
        assert!(report.markdown_summary.contains("Циклические зависимости"));
    }

    #[test]
    fn impact_walks_reverse_edges_and_rates_god_target_high() {
        let conn = setup_in_memory();
        GraphAnalytics::update_god_nodes(&conn, "p1").unwrap();

        let report = GraphAnalytics::analyze_impact(&conn, "p1", "hub.rs", 3).unwrap();
        assert_eq!(report.target_symbol, "hub.rs");
        assert_eq!(report.target_file.as_deref(), Some("src/hub.rs"));
        assert_eq!(report.risk_level, RiskLevel::High, "god target is High");
        assert!(report.risk_factors.iter().any(|f| f.contains("God Node")));
        assert_eq!(
            report.affected_nodes.len(),
            5,
            "mid, extra, cyc_a at depth 1; leaf, cyc_b at depth 2"
        );
        let leaf = report
            .affected_nodes
            .iter()
            .find(|n| n.label == "leaf.rs")
            .expect("leaf is a transitive consumer");
        assert_eq!(leaf.depth, 2);
        assert!(
            report
                .markdown_summary
                .contains("Обязательно согласуйте интерфейс")
        );
    }

    #[test]
    fn impact_rates_leaf_target_low_and_unknown_symbol_not_found() {
        let conn = setup_in_memory();

        let leaf = GraphAnalytics::analyze_impact(&conn, "p1", "leaf.rs", 3).unwrap();
        assert_eq!(leaf.affected_nodes.len(), 0, "leaf has no dependents");
        assert_eq!(leaf.risk_level, RiskLevel::Low);
        assert!(leaf.markdown_summary.contains("изолирован"));

        let lonely = GraphAnalytics::analyze_impact(&conn, "p1", "lonely.rs", 3).unwrap();
        assert_eq!(lonely.risk_level, RiskLevel::Low);

        let unknown = GraphAnalytics::analyze_impact(&conn, "p1", "no-such-symbol", 5).unwrap();
        assert!(unknown.affected_nodes.is_empty());
        assert_eq!(unknown.risk_level, RiskLevel::Low);
        assert!(
            unknown.risk_factors[0].contains("не найден"),
            "not-found reason is surfaced: {:?}",
            unknown.risk_factors
        );
    }

    #[test]
    fn impact_respects_max_depth_clamp() {
        let conn = setup_in_memory();

        // max_depth 0 clamps to 1: only direct importers of hub.rs.
        let shallow = GraphAnalytics::analyze_impact(&conn, "p1", "hub.rs", 0).unwrap();
        assert_eq!(shallow.max_depth, 1);
        assert_eq!(shallow.affected_nodes.len(), 3, "mid, extra, cyc_a only");
        assert!(!shallow.affected_nodes.iter().any(|n| n.label == "leaf.rs"));

        // Depth 2 reaches leaf.rs through mid.rs.
        let deep = GraphAnalytics::analyze_impact(&conn, "p1", "hub.rs", 2).unwrap();
        assert!(deep.affected_nodes.iter().any(|n| n.label == "leaf.rs"));
        assert_eq!(deep.affected_nodes.len(), 5);
    }

    #[test]
    fn circular_dependency_detector_finds_the_two_node_cycle_only() {
        let conn = setup_in_memory();
        let cycles = GraphAnalytics::find_circular_dependencies(&conn, "p1").unwrap();
        assert_eq!(cycles.len(), 1, "only cyc_a ⇄ cyc_b is a cycle");
        assert_eq!(cycles[0].length, 2);
        let joined = cycles[0].nodes.join("|");
        assert!(joined.contains("cyc_a.rs") && joined.contains("cyc_b.rs"));

        assert!(
            GraphAnalytics::find_circular_dependencies(&conn, "p2")
                .unwrap()
                .is_empty(),
            "acyclic project reports no cycles"
        );
    }

    #[test]
    fn coupling_metrics_follow_martin_stability_categories() {
        let conn = setup_in_memory();
        let metrics = GraphAnalytics::compute_coupling_metrics(&conn, "p1").unwrap();

        let find = |name: &str| metrics.iter().find(|m| m.component == name).unwrap();
        let hub = find("src/hub.rs");
        assert_eq!(hub.afferent_ca, 3);
        assert_eq!(hub.efferent_ce, 0);
        assert!((hub.instability).abs() < 1e-6);
        assert_eq!(hub.category, "Стабильный (Ядро)");

        let leaf = find("src/leaf.rs");
        assert_eq!(leaf.afferent_ca, 0);
        assert_eq!(leaf.efferent_ce, 1);
        assert!((leaf.instability - 1.0).abs() < 1e-6);
        assert_eq!(leaf.category, "Нестабильный (Лист)");

        let mid = find("src/mid.rs");
        assert!((mid.instability - 0.5).abs() < 1e-6);
        assert_eq!(mid.category, "Сбалансированный");

        // Sorted by ascending instability: hub (0.0) comes first.
        assert_eq!(metrics.first().unwrap().component, "src/hub.rs");
        assert!(!metrics.iter().any(|m| m.component == "src/lonely.rs"));
    }

    #[test]
    fn project_context_renders_hubs_and_optional_task_section() {
        let conn = setup_in_memory();
        GraphAnalytics::update_god_nodes(&conn, "p1").unwrap();

        let plain = GraphAnalytics::build_project_context(&conn, "p1", None).unwrap();
        assert!(plain.starts_with("<project_context id=\"p1\">"));
        assert!(plain.contains("Core Architecture Hubs"));
        assert!(plain.contains("hub.rs"));
        assert!(plain.ends_with("</project_context>"));
        assert!(!plain.contains("Relevant Subsystems"));

        let with_task = GraphAnalytics::build_project_context(&conn, "p1", Some("hub")).unwrap();
        assert!(with_task.contains("Relevant Subsystems for Task 'hub'"));
        assert!(with_task.contains("hub.rs"));
    }
}
