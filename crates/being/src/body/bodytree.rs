#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::platform::collections::{HashMap, HashSet};
use common::common_components::*;
use common::common_components::HashId;
use common::common_states::AssetLoading;
use common::def_db;
use common::log_targets::BODY_BUILD;
use game_common::game_common_components::{Templ, TemplEntiRef};
use ::being_shared::*;

use crate::body::body_hp_systems::UserBodypartInstances;
use crate::body::body_seris::BodypartNodeSeri;
use crate::body::bodypart::bodypart_resources::*;

#[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
#[require(
    AssetScoped,
    Prefix::trunc("BodyTree"),
)]
pub struct BodyTreeTemplate;

#[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
#[require(
    AssetScoped,
    Prefix::trunc("BodyTree"),
)]
pub struct BodyTreeAbstract;

common::define_entity_map_systems_no_replicate!(
    main_component: BodyTreeTemplate,
    with_filters: (With<Templ>, ),
    abbreviation: BodyTree,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "BodyTree",
    despawn_trigger: BodyTreeTemplate,
    id_type: common::common_components::StrId,
);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodyTreeSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_body_tree_template,
        ))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (
                (init_bodytree_templates, map_body_tree_template_id_to_entity)
                    .chain()
                    .in_set(BodyTreeSystems),
            ),
        )
    ;
}

#[derive(Debug)]
struct ParsedBodyTreeDef {
    id: String,
    rel_path: String,
    is_abstract: bool,
    root: BodypartNodeSeri,
}

#[derive(Debug)]
struct RawBodyTreeDef {
    id: String,
    rel_path: String,
    base_id: Option<String>,
    is_abstract: bool,
    root: Option<BodypartNodeSeri>,
    add_nodes: Vec<BodyTreeAddNodeSpec>,
    remove_paths: Vec<BodyTreePathSpec>,
    label_overrides: Vec<BodyTreeLabelOverrideSpec>,
    replace_nodes: Vec<BodyTreeReplaceNodeSpec>,
}

#[derive(Debug, Clone)]
struct BodyTreeAddNodeSpec {
    parent_path: BodyTreePathSpec,
    node: BodypartNodeSeri,
}

#[derive(Debug, Clone)]
struct BodyTreeAddNodeFrame {
    parent_path: Option<BodyTreePathSpec>,
    node: BodypartNodeSeri,
}

#[derive(Debug, Clone)]
struct BodyTreeLabelOverrideSpec {
    target_path: BodyTreePathSpec,
    label_override: String,
}

#[derive(Debug, Clone)]
struct BodyTreeReplaceNodeSpec {
    target_path: BodyTreePathSpec,
    node: BodypartNodeSeri,
}

#[derive(Debug, Clone)]
struct BodyTreePathSpec {
    segments: Vec<BodyTreePathSegment>,
}

#[derive(Debug, Clone)]
struct BodyTreePathSegment {
    part_id: String,
    nth: usize,
}

#[allow(unused_parens, )]
pub fn init_bodytree_templates(
    mut cmd: Commands,
    bodytree_map: Res<BodyTreeTemplateEntityMap>,
    part_map: Res<BodypartEntityMap>,
    part_hash_query: Query<&HashId, With<Bodypart>>,
) {
    if !bodytree_map.0.is_empty() {
        return;
    }

    let mut spawned_count = 0usize;
    for def in load_bodytree_defs() {
        let tree_id = match StrId::new_with_result(def.id, 3) {
            Ok(id) => id,
            Err(err) => {
                error!(target: BODY_BUILD, "Bodytree id parse failed in '{}': {}", def.rel_path, err);
                continue;
            }
        };
        let tree_ent = cmd.spawn((
            tree_id.clone(),
            AddHashIdFromStrId,
            HashId::from(tree_id.as_str()),
            DisplayName::trunc(tree_id.as_str()),
            BodyTreeTemplate,
            Templ,
        )).id();
        if def.is_abstract {
            cmd.entity(tree_ent).insert(BodyTreeAbstract);
        }

        let root_id = StrId::trunc(def.root.part_id.as_str());
        let root = rec_build_templ_body_tree_nodes(
            &mut cmd,
            &part_map,
            &part_hash_query,
            tree_ent,
            &tree_id,
            def.root,
            None,
        );
        let Some(root_ent) = root else {
            warn!(target: BODY_BUILD, "Bodytree '{}' root '{}' was not built; despawning invalid template", tree_id, root_id);
            cmd.entity(tree_ent).try_despawn();
            continue;
        };
        cmd.entity(root_ent).insert(TreeRoot);
        spawned_count += 1;
        trace!(target: BODY_BUILD, "Initialized bodytree template '{}' with root {}", tree_id, root_id);
    }
    if spawned_count > 0 {
        debug!(target: BODY_BUILD, "Initialized {} bodytree template entities", spawned_count);
    }
}

pub(crate) fn rec_build_templ_body_tree_nodes(
    cmd: &mut Commands,
    part_map: &BodypartEntityMap,
    part_hash_query: &Query<&HashId, With<Bodypart>>,
    templ_owner_ent: Entity,
    owner_id: &StrId,
    node: BodypartNodeSeri,
    parent_node_ent: Option<Entity>,
) -> Option<Entity> {
    let node_bodypart_id = StrId::trunc(node.part_id.as_str());
    let Ok(source_part_ent) = part_map.0.get_cloned(&node_bodypart_id) else {
        error!(target: BODY_BUILD, "Bodypart '{}' not found for bodytree/body '{}', skipping", node_bodypart_id, owner_id);
        return None;
    };

    let parent_bodypart = parent_node_ent.unwrap_or(templ_owner_ent);
    let node_ent = cmd.entity(source_part_ent).clone_and_spawn_with_opt_out(|builder| {
        builder.deny::<(
            Templ,
            ChildOf,
            Children,
            BodypartChildrenBodyparts,
        )>();
    }).id();
    let Ok(&source_part_hash) = part_hash_query.get(source_part_ent) else {
        error!(target: BODY_BUILD, "Bodypart '{}' has no HashId while building bodytree/body '{}', skipping", node_bodypart_id, owner_id);
        return None;
    };
    cmd.entity(node_ent).insert((
        BodypartChildOfBodypart { parent_bodypart },
        ChildOf(templ_owner_ent),
        TemplEntiRef(source_part_ent),
        TemplEntiHashIdRef(source_part_hash),
        UserBodypartInstances::default(),
        Templ,
        Name::default(),
    ));

    let override_label = node.label_override.trim();
    if !override_label.is_empty() {
        cmd.entity(node_ent).insert(DisplayName::trunc(override_label));
    }

    for child in node.children {
        rec_build_templ_body_tree_nodes(
            cmd,
            part_map,
            part_hash_query,
            templ_owner_ent,
            owner_id,
            child,
            Some(node_ent),
        );
    }

    Some(node_ent)
}

fn load_bodytree_defs() -> Vec<ParsedBodyTreeDef> {
    let mut discovered = match def_db::discover_assets_files_by_suffixes(&[".bodytree"]) {
        Ok(found) => found,
        Err(err) => {
            error!(target: BODY_BUILD, "Failed discovering .bodytree files: {err:#}");
            return Vec::new();
        }
    };
    discovered.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut raw_by_id: HashMap<String, RawBodyTreeDef> = HashMap::default();
    for (source, abs_path) in discovered {
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(content) => content,
            Err(err) => {
                error!(target: BODY_BUILD, "Failed reading bodytree '{}': {}", source.rel_path, err);
                continue;
            }
        };
        let parsed = match parse_bodytree_def(&content, &source.rel_path) {
            Ok(parsed) => parsed,
            Err(err) => {
                error!(target: BODY_BUILD, "Failed parsing bodytree '{}': {}", source.rel_path, err);
                continue;
            }
        };

        if let Some(prev) = raw_by_id.insert(parsed.id.clone(), parsed) {
            debug!(target: BODY_BUILD, "Bodytree '{}' overridden: '{}' -> '{}'", prev.id, prev.rel_path, source.rel_path);
            continue;
        }
    }

    let mut resolved = HashMap::<String, BodypartNodeSeri>::default();
    let mut resolving = HashSet::<String>::default();
    let mut ordered = Vec::new();
    ordered.reserve(raw_by_id.len());

    let mut raw_ids = Vec::with_capacity(raw_by_id.len());
    raw_ids.extend(raw_by_id.keys().cloned());
    raw_ids.sort();

    for id in raw_ids {
        let Some(raw) = raw_by_id.get(&id) else {
            continue;
        };
        let root = match resolve_bodytree_root(&id, &raw_by_id, &mut resolved, &mut resolving) {
            Ok(root) => root,
            Err(err) => {
                error!(target: BODY_BUILD, "Failed resolving bodytree '{}': {}", id, err);
                continue;
            }
        };
        ordered.push(ParsedBodyTreeDef {
            id: id.clone(),
            rel_path: raw.rel_path.clone(),
            is_abstract: raw.is_abstract,
            root,
        });
    }

    ordered
}

fn parse_bodytree_def(content: &str, rel_path: &str) -> Result<RawBodyTreeDef, String> {
    let mut id = None::<String>;
    let mut base_id = None::<String>;
    let mut is_abstract = false;
    let mut root_lines = Vec::new();
    let mut add_lines = Vec::new();
    let mut remove_lines = Vec::new();
    let mut label_lines = Vec::new();
    let mut replace_lines = Vec::new();
    let mut section = BodyTreeParseSection::Tree;
    let mut saw_inheritance = false;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line_without_comment = strip_inline_comment(raw_line);
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(raw_id) = trimmed.strip_prefix("id:")
            .or_else(|| trimmed.strip_prefix("id "))
        {
            let parsed = parse_text_value(raw_id);
            if parsed.is_empty() {
                return Err(format!("{rel_path}:{line_no} has empty id"));
            }
            id = Some(parsed);
            continue;
        }

        if let Some(raw_base) = trimmed
            .strip_prefix("extends:")
            .or_else(|| trimmed.strip_prefix("base:"))
        {
            let parsed = parse_text_value(raw_base);
            if parsed.is_empty() {
                return Err(format!("{rel_path}:{line_no} has empty extends/base id"));
            }
            base_id = Some(parsed);
            saw_inheritance = true;
            continue;
        }

        if let Some(raw_abstract) = trimmed.strip_prefix("abstract:") {
            let parsed = parse_text_value(raw_abstract);
            is_abstract = if parsed.is_empty() {
                true
            } else {
                match parsed.to_ascii_lowercase().as_str() {
                    "true" | "yes" | "on" => true,
                    "false" | "no" | "off" => false,
                    other => {
                        return Err(format!("{rel_path}:{line_no} has invalid abstract value '{other}'"));
                    }
                }
            };
            continue;
        }

        match trimmed {
            "tree:" | "root:" => {
                section = BodyTreeParseSection::Tree;
                continue;
            }
            "add:" => {
                section = BodyTreeParseSection::Add;
                saw_inheritance = true;
                continue;
            }
            "remove:" => {
                section = BodyTreeParseSection::Remove;
                saw_inheritance = true;
                continue;
            }
            "label:" | "labels:" | "rename:" => {
                section = BodyTreeParseSection::Label;
                saw_inheritance = true;
                continue;
            }
            "replace:" => {
                section = BodyTreeParseSection::Replace;
                saw_inheritance = true;
                continue;
            }
            _ => {}
        }

        match section {
            BodyTreeParseSection::Tree => root_lines.push((line_no, line_without_comment.to_string())),
            BodyTreeParseSection::Add => add_lines.push((line_no, line_without_comment.to_string())),
            BodyTreeParseSection::Remove => remove_lines.push((line_no, line_without_comment.to_string())),
            BodyTreeParseSection::Label => label_lines.push((line_no, line_without_comment.to_string())),
            BodyTreeParseSection::Replace => replace_lines.push((line_no, line_without_comment.to_string())),
        }
    }

    let id = id.unwrap_or_else(|| id_from_rel_path(rel_path));
    if id.trim().is_empty() {
        return Err(format!("{rel_path} has an empty id and fallback file stem is also empty"));
    }

    if !saw_inheritance {
        let root = parse_indented_bodytree_nodes(&root_lines, rel_path)?;
        return Ok(RawBodyTreeDef {
            id,
            rel_path: rel_path.to_string(),
            base_id: None,
            is_abstract,
            root: Some(root),
            add_nodes: Vec::new(),
            remove_paths: Vec::new(),
            label_overrides: Vec::new(),
            replace_nodes: Vec::new(),
        });
    }

    let add_nodes = parse_indented_bodytree_add_nodes(&add_lines, rel_path)?;

    let mut remove_paths = Vec::new();
    for (line_no, raw_line) in remove_lines {
        let line_body = strip_list_item_prefix(&raw_line);
        if line_body.is_empty() {
            continue;
        }
        remove_paths.push(parse_bodytree_path(line_body, rel_path, line_no)?);
    }

    let mut label_overrides = Vec::new();
    for (line_no, raw_line) in label_lines {
        let line_body = strip_list_item_prefix(&raw_line);
        if line_body.is_empty() {
            continue;
        }
        let (path_raw, label_raw) = split_path_and_optional_label(line_body);
        let target_path = parse_bodytree_path(path_raw, rel_path, line_no)?;
        let label_override = parse_lbl_value(label_raw);
        label_overrides.push(BodyTreeLabelOverrideSpec { target_path, label_override });
    }

    let mut replace_nodes = Vec::new();
    for (line_no, raw_line) in replace_lines {
        let line_body = strip_list_item_prefix(&raw_line);
        if line_body.is_empty() {
            continue;
        }
        let (path_raw, node_raw) = line_body
            .rsplit_once("->")
            .ok_or_else(|| format!("{rel_path}:{line_no} replace entry must contain a target path and node spec separated by '->'"))?;
        let target_path = parse_bodytree_path(path_raw, rel_path, line_no)?;
        let node = parse_bodytree_node_spec(node_raw, rel_path, line_no)?;
        replace_nodes.push(BodyTreeReplaceNodeSpec { target_path, node });
    }

    Ok(RawBodyTreeDef {
        id,
        rel_path: rel_path.to_string(),
        base_id,
        is_abstract,
        root: None,
        add_nodes,
        remove_paths,
        label_overrides,
        replace_nodes,
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BodyTreeParseSection {
    Tree,
    Add,
    Remove,
    Label,
    Replace,
}

fn resolve_bodytree_root(
    id: &str,
    raw_by_id: &HashMap<String, RawBodyTreeDef>,
    resolved: &mut HashMap<String, BodypartNodeSeri>,
    resolving: &mut HashSet<String>,
) -> Result<BodypartNodeSeri, String> {
    if let Some(root) = resolved.get(id) {
        return Ok(root.clone());
    }

    if !resolving.insert(id.to_string()) {
        return Err(format!("Bodytree '{}' has a cyclic inheritance chain", id));
    }

    let Some(raw) = raw_by_id.get(id) else {
        resolving.remove(id);
        return Err(format!("Bodytree '{}' not found while resolving inheritance", id));
    };

    let mut root = if let Some(base_id) = &raw.base_id {
        resolve_bodytree_root(base_id, raw_by_id, resolved, resolving)?
    } else {
        let Some(root) = raw.root.clone() else {
            resolving.remove(id);
            return Err(format!("Bodytree '{}' does not define a root tree", id));
        };
        root
    };

    for remove_path in &raw.remove_paths {
        apply_bodytree_remove(&mut root, remove_path, &raw.rel_path)?;
    }
    for add_node in &raw.add_nodes {
        apply_bodytree_add(&mut root, add_node)?;
    }
    for label_override in &raw.label_overrides {
        apply_bodytree_label_override(&mut root, label_override, &raw.rel_path)?;
    }
    for replace_node in &raw.replace_nodes {
        apply_bodytree_replace(&mut root, replace_node, &raw.rel_path)?;
    }

    resolving.remove(id);
    resolved.insert(id.to_string(), root.clone());
    Ok(root)
}

fn apply_bodytree_remove(
    root: &mut BodypartNodeSeri,
    remove_path: &BodyTreePathSpec,
    rel_path: &str,
) -> Result<(), String> {
    let Some((target_segment, parent_indices)) = remove_path.segments.split_last() else {
        return Err(format!("{rel_path} has an empty remove path"));
    };
    let parent = resolve_bodytree_node_mut(root, &BodyTreePathSpec {
        segments: parent_indices.to_vec(),
    })?;
    let Some(remove_idx) = nth_matching_child_index(&parent.children, target_segment) else {
        return Err(format!(
            "{rel_path} could not remove '{}': node not found",
            format_bodytree_path(remove_path)
        ));
    };
    parent.children.remove(remove_idx);
    Ok(())
}

fn apply_bodytree_add(
    root: &mut BodypartNodeSeri,
    add_node: &BodyTreeAddNodeSpec,
) -> Result<(), String> {
    let parent = resolve_bodytree_node_mut(root, &add_node.parent_path)?;
    parent.children.push(add_node.node.clone());
    Ok(())
}

fn apply_bodytree_label_override(
    root: &mut BodypartNodeSeri,
    label_override: &BodyTreeLabelOverrideSpec,
    rel_path: &str,
) -> Result<(), String> {
    let node = resolve_bodytree_node_mut(root, &label_override.target_path)?;
    if label_override.label_override.trim().is_empty() {
        return Err(format!("{rel_path} has an empty label override for '{}'", format_bodytree_path(&label_override.target_path)));
    }
    node.label_override = label_override.label_override.clone();
    Ok(())
}

fn apply_bodytree_replace(
    root: &mut BodypartNodeSeri,
    replace_node: &BodyTreeReplaceNodeSpec,
    rel_path: &str,
) -> Result<(), String> {
    let Some((target_segment, parent_indices)) = replace_node.target_path.segments.split_last() else {
        return Err(format!("{rel_path} has an empty replace path"));
    };
    let parent = resolve_bodytree_node_mut(root, &BodyTreePathSpec {
        segments: parent_indices.to_vec(),
    })?;
    let Some(replace_idx) = nth_matching_child_index(&parent.children, target_segment) else {
        return Err(format!(
            "{rel_path} could not replace '{}': node not found",
            format_bodytree_path(&replace_node.target_path)
        ));
    };
    parent.children[replace_idx] = replace_node.node.clone();
    Ok(())
}

fn parse_indented_bodytree_add_nodes(
    lines: &[(usize, String)],
    rel_path: &str,
) -> Result<Vec<BodyTreeAddNodeSpec>, String> {
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let mut first_depth = None::<usize>;
    for (line_no, raw_line) in lines {
        let (depth, body) = parse_indented_line(raw_line, *line_no, rel_path)?;
        if strip_list_item_prefix(body).is_empty() {
            continue;
        }
        first_depth = Some(depth);
        break;
    }

    let Some(base_depth) = first_depth else {
        return Ok(Vec::new());
    };

    let mut roots = Vec::new();
    let mut stack = Vec::new();
    stack.reserve(lines.len());

    for (line_no, raw_line) in lines {
        let (depth, node_body) = parse_indented_line(raw_line, *line_no, rel_path)?;
        let node_body = strip_list_item_prefix(node_body);
        if node_body.is_empty() {
            return Err(format!("{rel_path}:{line_no} has an empty bodytree node"));
        }
        if depth < base_depth {
            return Err(format!("{rel_path}:{line_no} has inconsistent add indentation"));
        }

        let relative_depth = depth - base_depth;
        while stack.len() > relative_depth {
            attach_last_bodytree_add_node(&mut stack, &mut roots);
        }

        let frame = if relative_depth == 0 {
            let (path_raw, node_raw) = node_body
                .rsplit_once('>')
                .ok_or_else(|| format!("{rel_path}:{line_no} add entry must contain a parent path and node spec separated by '>'"))?;
            let parent_path = parse_bodytree_path(path_raw, rel_path, *line_no)?;
            let node = parse_bodytree_node_spec(node_raw, rel_path, *line_no)?;
            BodyTreeAddNodeFrame {
                parent_path: Some(parent_path),
                node,
            }
        } else {
            let node = parse_bodytree_node_spec(node_body, rel_path, *line_no)?;
            BodyTreeAddNodeFrame {
                parent_path: None,
                node,
            }
        };
        stack.push(frame);
    }

    while !stack.is_empty() {
        attach_last_bodytree_add_node(&mut stack, &mut roots);
    }

    Ok(roots)
}

fn attach_last_bodytree_add_node(
    stack: &mut Vec<BodyTreeAddNodeFrame>,
    roots: &mut Vec<BodyTreeAddNodeSpec>,
) {
    let Some(frame) = stack.pop() else {
        return;
    };
    if let Some(parent) = stack.last_mut() {
        parent.node.children.push(frame.node);
        return;
    }
    let Some(parent_path) = frame.parent_path else {
        return;
    };
    roots.push(BodyTreeAddNodeSpec {
        parent_path,
        node: frame.node,
    });
}

fn attach_last_bodytree_node(
    stack: &mut Vec<BodypartNodeSeri>,
    roots: &mut Vec<BodypartNodeSeri>,
) {
    let Some(node) = stack.pop() else {
        return;
    };
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
        return;
    }
    roots.push(node);
}

fn resolve_bodytree_node_mut<'a>(
    root: &'a mut BodypartNodeSeri,
    path: &BodyTreePathSpec,
) -> Result<&'a mut BodypartNodeSeri, String> {
    if path.segments.is_empty() {
        return Ok(root);
    }

    let mut current = root;
    let mut start_idx = 0usize;
    if let Some(first) = path.segments.first()
        && first.part_id == current.part_id
        && first.nth == 0
    {
        start_idx = 1;
    }

    for segment in &path.segments[start_idx..] {
        let child_idx = nth_matching_child_index(&current.children, segment)
            .ok_or_else(|| format!("Bodytree path '{}' not found", format_bodytree_path(path)))?;
        current = current
            .children
            .get_mut(child_idx)
            .ok_or_else(|| format!("Bodytree path '{}' not found", format_bodytree_path(path)))?;
    }

    Ok(current)
}

fn nth_matching_child_index(
    children: &[BodypartNodeSeri],
    segment: &BodyTreePathSegment,
) -> Option<usize> {
    let mut seen = 0usize;
    for (idx, child) in children.iter().enumerate() {
        if child.part_id != segment.part_id {
            continue;
        }
        if seen == segment.nth {
            return Some(idx);
        }
        seen += 1;
    }
    None
}

fn parse_indented_bodytree_nodes(
    lines: &[(usize, String)],
    rel_path: &str,
) -> Result<BodypartNodeSeri, String> {
    if lines.is_empty() {
        return Err(format!("{rel_path} has no bodytree nodes"));
    }

    let mut roots = Vec::new();
    let mut stack = Vec::new();
    roots.reserve(2);
    stack.reserve(lines.len());
    for (line_no, raw_line) in lines {
        let (depth, node_body) = parse_indented_line(raw_line, *line_no, rel_path)?;
        while stack.len() > depth {
            attach_last_bodytree_node(&mut stack, &mut roots);
        }

        let node_body = strip_list_item_prefix(node_body);
        if node_body.is_empty() {
            return Err(format!("{rel_path}:{line_no} has an empty bodytree node"));
        }
        let node = parse_bodytree_node_spec(node_body, rel_path, *line_no)?;
        stack.push(node);
    }

    while !stack.is_empty() {
        attach_last_bodytree_node(&mut stack, &mut roots);
    }

    if roots.len() != 1 {
        return Err(format!("{rel_path} must define exactly one root node, found {}", roots.len()));
    }

    Ok(roots.pop().unwrap_or_default())
}

fn parse_indented_line<'a>(
    raw_line: &'a str,
    line_no: usize,
    rel_path: &str,
) -> Result<(usize, &'a str), String> {
    let bytes = raw_line.as_bytes();
    let mut idx = 0usize;
    let mut spaces = 0usize;
    while idx < bytes.len() {
        match bytes[idx] {
            b' ' => {
                spaces += 1;
                idx += 1;
            }
            b'\t' => {
                return Err(format!("{rel_path}:{line_no} uses tabs for indentation; use two-space indentation"));
            }
            _ => break,
        }
    }

    if spaces % 2 != 0 {
        return Err(format!("{rel_path}:{line_no} has odd indentation; expected multiples of 2 spaces"));
    }

    Ok((spaces / 2, &raw_line[idx..]))
}

fn strip_list_item_prefix(line: &str) -> &str {
    line.trim().strip_prefix("- ").unwrap_or(line.trim())
}

fn parse_bodytree_path(
    raw: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<BodyTreePathSpec, String> {
    let mut segments = Vec::new();
    for raw_segment in raw.split('>') {
        let segment = parse_bodytree_path_segment(raw_segment, rel_path, line_no)?;
        segments.push(segment);
    }
    if segments.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty bodytree path"));
    }
    Ok(BodyTreePathSpec { segments })
}

fn parse_bodytree_path_segment(
    raw: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<BodyTreePathSegment, String> {
    let token = raw.trim();
    if token.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty bodytree path segment"));
    }

    let mut part_id = token.to_string();
    let mut nth = 0usize;

    if let Some(open_idx) = token.rfind('[')
        && token.ends_with(']')
    {
        let idx_str = &token[open_idx + 1..token.len() - 1];
        nth = idx_str
            .parse::<usize>()
            .map_err(|_| format!("{rel_path}:{line_no} has invalid bodytree path occurrence index in '{token}'"))?;
        part_id = token[..open_idx].trim().to_string();
    } else if let Some(hash_idx) = token.rfind('#') {
        let idx_str = &token[hash_idx + 1..];
        nth = idx_str
            .parse::<usize>()
            .map_err(|_| format!("{rel_path}:{line_no} has invalid bodytree path occurrence index in '{token}'"))?;
        part_id = token[..hash_idx].trim().to_string();
    }

    if part_id.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty bodytree path segment"));
    }
    Ok(BodyTreePathSegment { part_id, nth })
}

fn parse_bodytree_node_spec(
    raw: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<BodypartNodeSeri, String> {
    let node_body = raw.trim();
    if node_body.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty bodytree node"));
    }
    let (part_raw, label_raw) = split_part_and_label(node_body);
    let part_id = parse_bodytree_part_id(part_raw);
    if part_id.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty part_id"));
    }
    let label_override = parse_lbl_value(label_raw);
    Ok(BodypartNodeSeri {
        part_id,
        label_override,
        children: Vec::new(),
    })
}

fn parse_bodytree_part_id(raw: &str) -> String {
    let token = parse_text_value(raw);
    if let Some(open_idx) = token.rfind('[')
        && token.ends_with(']')
    {
        if token[open_idx + 1..token.len() - 1].parse::<usize>().is_ok() {
            return token[..open_idx].trim().to_string();
        }
    } else if let Some(hash_idx) = token.rfind('#') {
        if token[hash_idx + 1..].parse::<usize>().is_ok() {
            return token[..hash_idx].trim().to_string();
        }
    }
    token
}

fn format_bodytree_path(path: &BodyTreePathSpec) -> String {
    let mut out = String::new();
    for (idx, segment) in path.segments.iter().enumerate() {
        if idx > 0 {
            out.push_str(" > ");
        }
        out.push_str(&segment.part_id);
        if segment.nth != 0 {
            out.push('[');
            out.push_str(&segment.nth.to_string());
            out.push(']');
        }
    }
    out
}

fn strip_inline_comment(line: &str) -> &str {
    let Some(comment_idx) = line.find('#') else {
        return line;
    };
    &line[..comment_idx]
}

fn parse_text_value(raw: &str) -> String {
    let raw = raw.trim().trim_end_matches(',').trim();
    if let Some(quoted) = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return quoted.to_string();
    }
    raw.to_string()
}

fn split_part_and_label(raw: &str) -> (&str, &str) {
    split_path_and_optional_label(raw)
}

fn split_path_and_optional_label(raw: &str) -> (&str, &str) {
    let trimmed = raw.trim();
    if let Some(lbl_idx) = trimmed.find(" lbl") {
        let part = trimmed[..lbl_idx].trim();
        let label = trimmed[lbl_idx + 1..].trim();
        return (part, label);
    }
    (trimmed, "")
}

fn parse_lbl_value(raw: &str) -> String {
    let raw = raw.trim();
    let Some(raw) = raw.strip_prefix("lbl") else {
        return String::new();
    };
    parse_text_value(raw)
}

fn id_from_rel_path(rel_path: &str) -> String {
    let file_name = rel_path.rsplit('/').next().unwrap_or_default();
    file_name
        .strip_suffix(".bodytree")
        .unwrap_or(file_name)
        .to_string()
}
