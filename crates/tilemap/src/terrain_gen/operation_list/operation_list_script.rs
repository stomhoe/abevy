use bevy::prelude::*;
use common::common_components::{HashId, StrId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::terrain_gen::operation_list::operation_list_resources::{OpListSeri, OpListSerisHandles};
use crate::terrain_gen::terrgen_expression::{Assignment, Expr, ExprOpList};

#[allow(unused_parens)]
pub fn load_tg_oplists(
    mut seris_handles: Option<ResMut<OpListSerisHandles>>,
    mut assets: Option<ResMut<Assets<OpListSeri>>>,
) {
    let (Some(mut seris_handles), Some(mut assets)) = (seris_handles.take(), assets.take()) else {
        return;
    };

    let Some(root_dir) = resolve_oplist_root_dir() else {
        return;
    };

    let mut files = Vec::new();
    collect_tg_files(&root_dir, &mut files);
    if files.is_empty() {
        return;
    }
    files.sort();

    let mut compiled_by_id: BTreeMap<String, OpListSeri> = BTreeMap::new();
    for file in files {
        let source = match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(err) => {
                error!(target: "oplist_tg", "Failed reading '{}': {}", file.display(), err);
                continue;
            }
        };

        let compiled = match parse_tg_script_to_expr_tree(&source, &file) {
            Ok((id, roots, tags, debug_vars, size, bifs, expr_tree)) => OpListSeri {
                id,
                tags,
                debug_vars,
                root_in_dimensions: roots,
                bifs,
                size,
                expr_tree,
            },
            Err(err) => {
                error!(target: "oplist_tg", "Failed parsing '{}': {}", file.display(), err);
                continue;
            }
        };

        if compiled_by_id.insert(compiled.id.clone(), compiled).is_some() {
            warn!(
                target: "oplist_tg",
                "Duplicate TG script id found; later file overrides id '{}'",
                file.display()
            );
        }
    }

    if compiled_by_id.is_empty() {
        return;
    }

    // TG scripts are the canonical oplist source in the new system.
    // Drop previously tracked handles (including legacy .ron variants)
    // so duplicate ids cannot resolve to stale assets.
    seris_handles.handles.clear();

    let mut added = 0usize;
    for (_, seri) in compiled_by_id {
        let handle = assets.add(seri);
        seris_handles.handles.push(handle);
        added += 1;
    }

    info!(target: "oplist_tg", "Loaded {} TG oplist script(s)", added);
}

fn resolve_oplist_root_dir() -> Option<PathBuf> {
    let rel = Path::new("assets")
        .join("ron")
        .join("tilemap")
        .join("terrgen")
        .join("oplist_scripts");

    if rel.is_dir() {
        return Some(rel);
    }

    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = Path::new(&manifest).join("assets").join("ron").join("tilemap").join("terrgen").join("oplist_scripts");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }

    warn!(
        target: "oplist_tg",
        "Could not locate oplist_scripts directory under assets/ron/tilemap/terrgen/oplist_scripts"
    );
    None
}

fn collect_tg_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tg_files(&path, out);
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".oplist.tg")
        {
            out.push(path);
        }
    }
}

pub fn parse_tg_script_to_expr_tree(
    source: &str,
    path: &Path,
) -> Result<(String, Vec<String>, Option<Vec<String>>, Vec<String>, Option<[u32; 2]>, Vec<(String, Vec<String>)>, ExprOpList), String> {
    let mut id: Option<String> = None;
    let mut tags: Option<Vec<String>> = None;
    let mut debug_vars: Vec<String> = Vec::new();
    let mut roots: Option<Vec<String>> = None;
    let mut size: Option<[u32; 2]> = None;
    let mut bifs: Vec<(String, Vec<String>)> = Vec::new();
    let mut assignments: Vec<Assignment> = Vec::new();
    let mut output_expr: Option<Expr> = None;

    let mut aliases: HashMap<String, String> = HashMap::new();
    aliases.insert("out".to_string(), "out".to_string());

    let mut in_block = false;
    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let stripped = strip_comments(raw_line, &mut in_block);
        let line = stripped.trim().trim_end_matches(';').trim();
        if line.is_empty() {
            continue;
        }

        if let Some(value) = key_value(line, "id") {
            id = Some(trim_token(value).to_string());
            continue;
        }
        if let Some(value) = key_value(line, "root_in_dimensions")
            .or_else(|| key_value(line, "roots"))
            .or_else(|| key_value(line, "root"))
        {
            roots = Some(parse_string_list(value));
            continue;
        }
        if let Some(value) = key_value(line, "tags") {
            tags = Some(parse_string_list(value));
            continue;
        }
        if let Some(value) = key_value(line, "debug")
            .or_else(|| key_value(line, "debug_vars"))
        {
            debug_vars = parse_string_list(value);
            continue;
        }
        if let Some(value) = key_value(line, "size") {
            size = Some(parse_size(value).ok_or_else(|| {
                format!(
                    "{}:{} invalid size '{}', expected like '(1,1)' or '1,1'",
                    path.display(),
                    line_no,
                    value
                )
            })?);
            continue;
        }

        if let Some(rest) = line.strip_prefix("alias ") {
            let (name, target) = rest.split_once('=').ok_or_else(|| {
                format!("{}:{} invalid alias syntax", path.display(), line_no)
            })?;
            aliases.insert(name.trim().to_string(), target.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("bif ") {
            let (branch_raw, tiles_raw) = rest.split_once("->").ok_or_else(|| {
                format!("{}:{} invalid bif syntax, expected 'bif <oplist> -> [tiles]'", path.display(), line_no)
            })?;
            let branch = trim_token(branch_raw.trim()).to_string();
            let tiles = parse_string_list(tiles_raw.trim());
            bifs.push((branch, tiles));
            continue;
        }

        if let Some((lhs_raw, rhs_raw)) = line.split_once('=') {
            let mut lhs = lhs_raw.trim();
            if let Some(rest) = lhs.strip_prefix("let ") {
                lhs = rest.trim();
            }
            if lhs.starts_with('$') {
                return Err(format!("{}:{} slot references are no longer supported; use named variables", path.display(), line_no));
            }

            let var_name = if lhs.eq_ignore_ascii_case("out") {
                "out".to_string()
            } else {
                lhs.to_string()
            };

            let expr = parse_expr_from_string(rhs_raw.trim(), &aliases, path, line_no)?;

            if var_name == "out" {
                output_expr = Some(expr);
            } else {
                assignments.push(Assignment {
                    name: StrId::trunc(var_name.clone()),
                    expr,
                });
                aliases.insert(var_name.clone(), var_name);
            }
            continue;
        }

        return Err(format!("{}:{} unrecognized statement '{}'", path.display(), line_no, line));
    }

    let id = id.ok_or_else(|| format!("{}: missing 'id'", path.display()))?;
    let root_in_dimensions = roots.unwrap_or_else(|| vec![String::new()]);
    let output = output_expr.ok_or_else(|| format!("{}: missing 'out' assignment", path.display()))?;

    Ok((
        id,
        root_in_dimensions,
        tags,
        debug_vars,
        size,
        bifs,
        ExprOpList {
            assignments,
            output,
        },
    ))
}

fn parse_expr_from_string(
    expr_str: &str,
    aliases: &HashMap<String, String>,
    path: &Path,
    line_no: usize,
) -> Result<Expr, String> {
    let expr = strip_enclosing_parentheses(expr_str.trim().trim_end_matches(';').trim());

    if let Some(arith_expr) = try_parse_inline_arithmetic(expr, aliases, path, line_no)? {
        return Ok(arith_expr);
    }

    if let Some(rest) = strip_comp_prefix(expr) {
        let operand = parse_expr_from_string(rest, aliases, path, line_no)?;
        return Ok(Expr::Complement { value: Box::new(operand) });
    }

    if !expr.contains('(') && !expr.contains(' ') {
        return build_operand_expr(expr, aliases, path, line_no);
    }

    let (op_raw, args_raw) = if let Some(open_idx) = expr.find('(') {
        let close_idx = expr.rfind(')').ok_or_else(|| {
            format!("{}:{} missing ')' in expression '{}'", path.display(), line_no, expr)
        })?;
        let op = expr[..open_idx].trim();
        let args = expr[open_idx + 1..close_idx].trim();
        (op, args)
    } else {
        let (op, rest) = expr.split_once(' ').ok_or_else(|| {
            format!(
                "{}:{} expected expression like op(arg1,arg2) or 'op arg1,arg2' (got '{}')",
                path.display(),
                line_no,
                expr
            )
        })?;
        (op.trim(), rest.trim())
    };

    let operation = normalize_operation(op_raw).ok_or_else(|| {
        format!("{}:{} unknown operation '{}'", path.display(), line_no, op_raw)
    })?;

    let mut operands = Vec::new();
    for arg in split_csv_like(args_raw) {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }
        let operand_expr = parse_expr_from_string(arg, aliases, path, line_no)?;
        operands.push(operand_expr);
    }

    build_expression_tree(operation, operands)
}

fn strip_enclosing_parentheses(mut expr: &str) -> &str {
    loop {
        let trimmed = expr.trim();
        if trimmed.len() >= 2 && trimmed.starts_with('(') && trimmed.ends_with(')') {
            let mut depth = 0i32;
            let mut valid = true;
            for (idx, ch) in trimmed.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 && idx != trimmed.len() - 1 {
                            valid = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if valid && depth == 0 {
                expr = &trimmed[1..trimmed.len() - 1];
                continue;
            }
        }
        return trimmed;
    }
}

fn strip_comp_prefix(expr: &str) -> Option<&str> {
    let trimmed = expr.trim_start();
    if let Some(rest) = trimmed.strip_prefix('!') {
        return Some(rest.trim_start());
    }

    let upper = trimmed.to_ascii_uppercase();
    for prefix in ["COMPL", "COMP"] {
        if upper.starts_with(prefix) {
            let rest = &trimmed[prefix.len()..];
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.starts_with('(') || !rest_trimmed.is_empty() {
                return Some(rest_trimmed);
            }
        }
    }
    None
}

fn normalize_operation(op: &str) -> Option<&'static str> {
    let op = op.trim().to_ascii_lowercase();
    match op.as_str() {
        "+" | "add" => Some("+"),
        "-" | "sub" | "subtract" => Some("-"),
        "*" | "mul" | "multiply" => Some("*"),
        "/" | "div" | "divide" => Some("/"),
        "*opo" | "opo" | "multiplyopo" | "multiply_opo" => Some("*opo"),
        "min" => Some("min"),
        "max" => Some("max"),
        "avg" | "average" => Some("avg"),
        "abs" => Some("abs"),
        "*nm" | "mulnorm" | "multiplynormalized" => Some("*nm"),
        "*nmabs" | "mulnormabs" | "multiplynormalizedabs" => Some("*nmabs"),
        "idxmax" | "imax" => Some("idxmax"),
        "idxnorm" | "inorm" => Some("idxnorm"),
        "lin" | "linear" => Some("lin"),
        "clamp" => Some("clamp"),
        _ => None,
    }
}

fn build_operand_expr(
    operand: &str,
    aliases: &HashMap<String, String>,
    path: &Path,
    line_no: usize,
) -> Result<Expr, String> {
    let operand = operand.trim();

    let (complement, base) = if let Some(base) = strip_comp_prefix(operand) {
        (true, base)
    } else {
        (false, operand)
    };

    if let Ok(value) = base.parse::<f32>() {
        let expr = Expr::Literal(value);
        return Ok(if complement { Expr::Complement { value: Box::new(expr) } } else { expr });
    }

    if base.starts_with('$') {
        return Err(format!("{}:{} slot references are no longer supported; use named variables", path.display(), line_no));
    }

    if let Some(name) = aliases.get(base) {
        let expr = Expr::Variable { name: StrId::trunc(name.clone()) };
        return Ok(if complement { Expr::Complement { value: Box::new(expr) } } else { expr });
    }

    if let Some(seed_str) = base.strip_prefix("hp") {
        let seed = seed_str.parse::<u64>().unwrap_or(1000);
        return Ok(Expr::HashPos { seed });
    }

    if let Some(pd_str) = base.strip_prefix("pd") {
        if pd_str.len() >= 2 {
            let (min_dist_str, seed_str) = pd_str.split_at(1);
            if let (Ok(min_dist), Ok(seed)) = (min_dist_str.parse::<u8>(), seed_str.parse::<u64>()) {
                return Ok(Expr::PoissonDisk { min_dist, seed });
            }
        }
    }

    if let Some(ent_str) = base.strip_prefix("fnl.") {
        let (noise_sample_range, ent_str) = if let Some(stripped) = ent_str.strip_prefix("1-1.") {
            (fnl::NoiseSampleRange::NegOneToOne, stripped)
        } else {
            (fnl::NoiseSampleRange::ZeroToOne, ent_str)
        };

        let (base_str, extra_seed) = if let Some(idx) = ent_str.rfind(".s") {
            let (base, seed_str) = ent_str.split_at(idx);
            let seed = seed_str[2..].parse::<i32>().unwrap_or(0);
            (base, seed)
        } else {
            (ent_str, 0)
        };

        return Ok(Expr::NoiseByName {
            name: HashId::from(base_str),
            sample_range: noise_sample_range,
            complement,
            seed_offset: extra_seed,
        });
    }

    let expr = Expr::Variable { name: StrId::trunc(base.to_string()) };
    Ok(if complement { Expr::Complement { value: Box::new(expr) } } else { expr })
}

fn build_expression_tree(operation: &str, operands: Vec<Expr>) -> Result<Expr, String> {
    match operation {
        "+" => fold_binary(operands, "Add"),
        "-" => fold_binary(operands, "Subtract"),
        "*" => fold_binary(operands, "Multiply"),
        "/" => fold_binary(operands, "Divide"),
        "*opo" => {
            if operands.is_empty() {
                return Err("MultiplyOpo requires at least 1 operand".to_string());
            }
            if operands.len() == 1 {
                return Ok(Expr::Complement {
                    value: Box::new(operands[0].clone()),
                });
            }
            let mut result = operands[0].clone();
            for operand in &operands[1..] {
                result = Expr::Multiply {
                    left: Box::new(result),
                    right: Box::new(Expr::Complement { value: Box::new(operand.clone()) }),
                };
            }
            Ok(result)
        }
        "min" => Ok(Expr::Min { values: operands }),
        "max" => Ok(Expr::Max { values: operands }),
        "avg" => Ok(Expr::Average { values: operands }),
        "abs" => {
            if operands.is_empty() {
                return Err("Abs requires an operand".to_string());
            }
            Ok(Expr::Abs { value: Box::new(operands[0].clone()) })
        }
        "*nm" => {
            if operands.len() < 2 {
                return Err("MultiplyNormalized requires 2 operands".to_string());
            }
            Ok(Expr::MultiplyNormalized {
                left: Box::new(operands[0].clone()),
                right: Box::new(operands[1].clone()),
            })
        }
        "*nmabs" => {
            if operands.len() < 2 {
                return Err("MultiplyNormalizedAbs requires 2 operands".to_string());
            }
            Ok(Expr::MultiplyNormalizedAbs {
                left: Box::new(operands[0].clone()),
                right: Box::new(operands[1].clone()),
            })
        }
        "idxmax" => Ok(Expr::IndexMax { values: operands }),
        "idxnorm" => {
            if operands.len() < 2 {
                return Err("IndexNorm requires 2 operands".to_string());
            }
            Ok(Expr::IndexNorm {
                value: Box::new(operands[0].clone()),
                multiplier: Box::new(operands[1].clone()),
            })
        }
        "lin" => Ok(Expr::Linear { values: operands }),
        "clamp" => {
            if operands.len() < 3 {
                return Err("Clamp requires 3 operands (value, min, max)".to_string());
            }
            Ok(Expr::Clamp {
                value: Box::new(operands[0].clone()),
                min: Box::new(operands[1].clone()),
                max: Box::new(operands[2].clone()),
            })
        }
        _ => Err(format!("Unknown operation: {}", operation)),
    }
}

fn fold_binary(operands: Vec<Expr>, op: &str) -> Result<Expr, String> {
    if operands.len() < 2 {
        return Err(format!("{} requires at least 2 operands", op));
    }
    let mut result = operands[0].clone();
    for operand in &operands[1..] {
        result = match op {
            "Add" => Expr::Add {
                left: Box::new(result),
                right: Box::new(operand.clone()),
            },
            "Subtract" => Expr::Subtract {
                left: Box::new(result),
                right: Box::new(operand.clone()),
            },
            "Multiply" => Expr::Multiply {
                left: Box::new(result),
                right: Box::new(operand.clone()),
            },
            "Divide" => Expr::Divide {
                left: Box::new(result),
                right: Box::new(operand.clone()),
            },
            _ => unreachable!(),
        };
    }
    Ok(result)
}

fn try_parse_inline_arithmetic(
    expr: &str,
    aliases: &HashMap<String, String>,
    path: &Path,
    line_no: usize,
) -> Result<Option<Expr>, String> {
    let expr = expr.trim();
    let mut depth = 0i32;
    let mut op_at: Option<(usize, char)> = None;
    for (idx, ch) in expr.char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => depth -= 1,
            '+' | '-' | '*' | '/' if depth == 0 => {
                if idx == 0 {
                    continue;
                }
                op_at = Some((idx, ch));
                break;
            }
            _ => {}
        }
    }

    let (idx, op) = match op_at {
        Some(found) => found,
        None => return Ok(None),
    };

    let left = expr[..idx].trim();
    let right = expr[idx + 1..].trim();
    if left.is_empty() || right.is_empty() {
        return Ok(None);
    }

    let left_expr = parse_expr_from_string(left, aliases, path, line_no)?;
    let right_expr = parse_expr_from_string(right, aliases, path, line_no)?;

    let result = match op {
        '*' => Expr::Multiply {
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        },
        '/' => Expr::Divide {
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        },
        '+' => Expr::Add {
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        },
        '-' => Expr::Subtract {
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        },
        _ => return Ok(None),
    };
    Ok(Some(result))
}

fn key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let key_colon = format!("{key}:");
    if let Some(rest) = line.strip_prefix(&key_colon) {
        return Some(rest.trim());
    }
    let key_space = format!("{key} ");
    if let Some(rest) = line.strip_prefix(&key_space) {
        return Some(rest.trim());
    }
    None
}

fn parse_size(value: &str) -> Option<[u32; 2]> {
    let mut nums = value
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u32>().ok());
    let w = nums.next()?;
    let h = nums.next()?;
    Some([w, h])
}

fn parse_string_list(input: &str) -> Vec<String> {
    let mut raw = input.trim();
    if raw.starts_with('[') && raw.ends_with(']') && raw.len() >= 2 {
        raw = &raw[1..raw.len() - 1];
    } else if raw.starts_with('(') && raw.ends_with(')') && raw.len() >= 2 {
        raw = &raw[1..raw.len() - 1];
    }

    let mut out = Vec::new();
    for token in split_csv_like(raw) {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(trim_token(trimmed).to_string());
    }
    out
}

fn split_csv_like(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut depth = 0i32;

    for ch in input.chars() {
        match ch {
            '"' | '\'' => {
                if let Some(q) = quote {
                    if q == ch {
                        quote = None;
                    }
                } else {
                    quote = Some(ch);
                }
                current.push(ch);
            }
            '(' | '[' if quote.is_none() => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' if quote.is_none() => {
                depth -= 1;
                current.push(ch);
            }
            ',' if quote.is_none() && depth == 0 => {
                out.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out
}

fn trim_token(token: &str) -> &str {
    let t = token.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        &t[1..t.len().saturating_sub(1)]
    } else {
        t
    }
}

fn strip_comments(line: &str, in_block: &mut bool) -> String {
    let mut out = String::new();
    let mut quote: Option<char> = None;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if *in_block {
            if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                *in_block = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                if let Some(q) = quote {
                    if q == c {
                        quote = None;
                    }
                } else {
                    quote = Some(c);
                }
                out.push(c);
                i += 1;
            }
            '/' if quote.is_none() && i + 1 < chars.len() => {
                match chars[i + 1] {
                    '/' => break,
                    '*' => {
                        *in_block = true;
                        i += 2;
                    }
                    _ => {
                        out.push(c);
                        i += 1;
                    }
                }
            }
            '#' if quote.is_none() => break,
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}
