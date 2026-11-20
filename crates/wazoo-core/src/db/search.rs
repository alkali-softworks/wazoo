/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Search Query Parser & SQL Builder
 *
 * Parses comma-separated positive/negative clauses (`not`, `!`), start/end anchors (`^`, `$`),
 * wildcards (`*`), LIKE escaping, and folder scopes into parameterized SQLite queries.
 */

use super::Database;
use crate::models::VideoRecord;
use rusqlite::Result;

struct ParsedClause {
    is_not: bool,
    has_start_anchor: bool,
    has_end_anchor: bool,
    words: String,
    is_path_prefix: bool,
}

fn escape_like_literal(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn parse_clause(clause: &str) -> Option<ParsedClause> {
    let trimmed = clause.trim();
    let (is_not, term) = if trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("not ") {
        (true, trimmed[4..].trim())
    } else if let Some(rest) = trimmed.strip_prefix('!') {
        (true, rest.trim())
    } else {
        (false, trimmed)
    };

    if term.is_empty() {
        return None;
    }

    let has_start_anchor = term.starts_with('^') && !term.starts_with("^^");
    let has_end_anchor = term.len() > 1 && term.ends_with('$') && !term.ends_with("$$");

    let core_term = match (has_start_anchor, has_end_anchor) {
        (true, true) => {
            if term.len() <= 2 {
                ""
            } else {
                &term[1..term.len() - 1]
            }
        }
        (true, false) => &term[1..],
        (false, true) => &term[..term.len() - 1],
        (false, false) => term,
    };

    if core_term.is_empty() {
        return None;
    }

    // Escape SQL LIKE special characters: '\', '%', and '_'
    // Also support '*' as an unescaped SQL wildcard '%'
    let escaped_term = escape_like_literal(core_term).replace('*', "%");

    // Match original wazoo-js: replace '-' with space, split words, join with '%'
    let words = escaped_term
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("%");

    let is_path_prefix = has_start_anchor
        && (core_term.starts_with('/')
            || core_term.starts_with('\\')
            || (core_term.len() >= 2 && core_term.as_bytes()[1] == b':'));

    Some(ParsedClause {
        is_not,
        has_start_anchor,
        has_end_anchor,
        words,
        is_path_prefix,
    })
}

fn push_param(params: &mut Vec<String>, value: String) -> usize {
    params.push(value);
    params.len()
}

fn like_pred(col: &str, op: &str, param_idx: usize) -> String {
    format!("{col} {op} ?{param_idx} ESCAPE '\\'")
}

fn join_clause_parts(is_not: bool, mut parts: Vec<String>, meta_cols: &[&str], meta_param: usize) -> String {
    let op = if is_not { "NOT LIKE" } else { "LIKE" };
    for col in meta_cols {
        if is_not {
            parts.push(format!("({col} IS NULL OR {col} {op} ?{meta_param} ESCAPE '\\')"));
        } else {
            parts.push(like_pred(col, op, meta_param));
        }
    }
    let joiner = if is_not { " AND " } else { " OR " };
    format!("({})", parts.join(joiner))
}

fn build_clause_condition(
    parsed: &ParsedClause,
    include_metadata: bool,
    params: &mut Vec<String>,
) -> String {
    let op = if parsed.is_not { "NOT LIKE" } else { "LIKE" };

    if parsed.is_path_prefix {
        let pattern = if parsed.has_end_anchor {
            parsed.words.clone()
        } else {
            format!("{}%", parsed.words)
        };
        let p_idx = push_param(params, pattern);
        return like_pred("v.path", op, p_idx);
    }

    match (parsed.has_start_anchor, parsed.has_end_anchor) {
        (true, true) => {
            // Exact anchor: ^term$
            let p1 = push_param(params, parsed.words.clone());
            let p2 = push_param(params, format!("%/{}.%", parsed.words));
            let p3 = push_param(params, format!("%\\\\{}.%", parsed.words));
            let parts = vec![
                like_pred("v.name", op, p1),
                like_pred("v.path", op, p2),
                like_pred("v.path", op, p3),
            ];
            let meta_cols: &[&str] = if include_metadata { &["m.title"] } else { &[] };
            join_clause_parts(parsed.is_not, parts, meta_cols, p1)
        }
        (true, false) => {
            // Prefix anchor: ^term
            let p1 = push_param(params, format!("{}%", parsed.words));
            let p2 = push_param(params, format!("%/{}%", parsed.words));
            let p3 = push_param(params, format!("%\\\\{}%", parsed.words));
            let parts = vec![
                like_pred("v.name", op, p1),
                like_pred("v.path", op, p2),
                like_pred("v.path", op, p3),
            ];
            let meta_cols: &[&str] = if include_metadata { &["m.title"] } else { &[] };
            join_clause_parts(parsed.is_not, parts, meta_cols, p1)
        }
        (false, true) => {
            // Suffix anchor: term$
            let p1 = push_param(params, format!("%{}", parsed.words));
            let p2 = push_param(params, format!("%{}.%", parsed.words));
            let parts = vec![
                like_pred("v.name", op, p1),
                like_pred("v.path", op, p2),
                like_pred("v.path", op, p1),
            ];
            let meta_cols: &[&str] = if include_metadata { &["m.title"] } else { &[] };
            join_clause_parts(parsed.is_not, parts, meta_cols, p1)
        }
        (false, false) => {
            // Standard unanchored search matching path, name, and optionally rich metadata
            let p_idx = push_param(params, format!("%{}%", parsed.words));
            let parts = vec![
                like_pred("v.path", op, p_idx),
                like_pred("v.name", op, p_idx),
            ];
            let meta_cols: &[&str] = if !include_metadata {
                &[]
            } else if parsed.is_not {
                &["m.genres", "m.title", "m.series_title", "m.directors"]
            } else {
                &[
                    "m.title",
                    "m.series_title",
                    "m.genres",
                    "CAST(m.year AS TEXT)",
                    "m.directors",
                    "m.actors",
                    "m.summary",
                ]
            };
            join_clause_parts(parsed.is_not, parts, meta_cols, p_idx)
        }
    }
}

fn append_folder_conditions(
    folders: &[String],
    where_conditions: &mut Vec<String>,
    params: &mut Vec<String>,
) {
    let active_folders: Vec<&String> = folders
        .iter()
        .filter(|f| !crate::i18n::is_all_folder(f))
        .collect();

    if active_folders.is_empty() {
        return;
    }

    let mut folder_conditions: Vec<String> = Vec::new();
    for folder in active_folders {
        if folder == "Misc" {
            folder_conditions.push("v.folder = 'Misc'".to_string());
        } else {
            let escaped_folder = escape_like_literal(folder);
            let p_idx = push_param(params, format!("{escaped_folder}%"));
            folder_conditions.push(like_pred("v.path", "LIKE", p_idx));
        }
    }
    where_conditions.push(format!("({})", folder_conditions.join(" OR ")));
}

impl Database {
    pub fn search_videos(&self, query_str: &str, folders: &[String]) -> Result<Vec<VideoRecord>> {
        self.search_videos_with_options(query_str, folders, true)
    }

    pub fn search_videos_with_options(
        &self,
        query_str: &str,
        folders: &[String],
        include_metadata: bool,
    ) -> Result<Vec<VideoRecord>> {
        let mut where_conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<String> = Vec::new();

        let mut positive_conditions: Vec<String> = Vec::new();
        let mut negative_conditions: Vec<String> = Vec::new();

        for clause in query_str
            .split(',')
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
        {
            let Some(parsed) = parse_clause(clause) else {
                continue;
            };
            let cond = build_clause_condition(&parsed, include_metadata, &mut param_values);
            if parsed.is_not {
                negative_conditions.push(cond);
            } else {
                positive_conditions.push(cond);
            }
        }

        // Positive clauses are grouped with OR: at least one positive term must match
        if !positive_conditions.is_empty() {
            where_conditions.push(format!("({})", positive_conditions.join(" OR ")));
        }

        // Negative clauses are grouped with AND: all negative terms must NOT match
        if !negative_conditions.is_empty() {
            where_conditions.push(format!("({})", negative_conditions.join(" AND ")));
        }

        append_folder_conditions(folders, &mut where_conditions, &mut param_values);

        let from_clause = if include_metadata {
            "FROM Video v LEFT JOIN VideoMetadata m ON v.id = m.video_id"
        } else {
            "FROM Video v"
        };

        let query = if where_conditions.is_empty() {
            format!(
                "SELECT DISTINCT v.id, v.name, v.path, v.folder {from_clause} ORDER BY v.path ASC"
            )
        } else {
            format!(
                "SELECT DISTINCT v.id, v.name, v.path, v.folder {from_clause} WHERE {} ORDER BY v.path ASC",
                where_conditions.join(" AND ")
            )
        };

        let mut stmt = self.conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(param_refs.as_slice(), Self::map_video_row)?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
