use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::common::*;
use ::game_common::*;
use ::sprite_shared::*;

#[allow(unused_parens)]
pub fn clone_templ_children_ents_for_new_instances(
    mut cmd: Commands,
    missing_clonechildren_query: Query<
        (Entity, &TemplEntiRef, Option<&Children>),
        (Changed<TemplEntiRef>, AnyDisabling, ),
    >,
    name_query: Query<&Name>,
    templ: Query<(&Children, Option<&HeldSprites>,), (AnyDisabling, With<CloneTemplChildren>)>,
    child_visibility_query: Query<&Visibility>,
    clone_children_query: Query<(), (AnyDisabling, With<CloneTemplChildren>)>,
) {
    let mut new_child_of = Vec::new();
    let mut new_base_holder_ref = Vec::new();
    let mut new_cloned_visibility = Vec::new();
    let mut clone_queue = Vec::new();

    missing_clonechildren_query.iter().for_each(|(new_ent, templ_ref, prev_children)| {
        
        clone_queue.clear();
        clone_queue.push((templ_ref.0, new_ent));
        
        while let Some((source_parent, clone_parent)) = clone_queue.pop() {
            let Ok((templ_children, templ_held_sprites)) = templ.get(source_parent) else {
                continue;
            };
            if let Some(prev_children) = prev_children {
                let name = name_query.get(new_ent).map(|name| name.as_str()).unwrap_or("<unnamed>");
                debug!(target: "entity_zero", "Entity {:?} ('{}') has CloneTemplChildren but already has children, despawning them first", new_ent, name);
                
                for child in prev_children.iter() {
                    cmd.entity(child).try_despawn();
                }
            }

            templ_children.iter().for_each(|child_to_clone| {
                let has_visibility = child_visibility_query.get(child_to_clone).is_ok();
                let child_has_clone_children = clone_children_query.get(child_to_clone).is_ok();
                let cloned_child = cmd.entity(child_to_clone).clone_and_spawn_with_opt_out(
                    move |builder| {
                        builder.deny::<DenyForTemplClonedChildren>();
                        builder.deny::<Replicated>();
                    }
                ).id();
                if has_visibility {
                    new_cloned_visibility.push((cloned_child, Visibility::Inherited));
                }
                new_child_of.push((cloned_child, (ChildOf(clone_parent), TemplEntiRef(child_to_clone))));

                debug!(target: "entity_zero", "Cloned child {:?} of EntityZero {:?} as child of {:?}", cloned_child, source_parent, clone_parent);

                if let Some(templ_held_sprites) = templ_held_sprites {
                    if templ_held_sprites.contains(&child_to_clone) {
                        new_base_holder_ref.push((cloned_child, BaseHolderRef { base: clone_parent }));
                    }
                }

                if child_has_clone_children {
                    clone_queue.push((child_to_clone, cloned_child));
                }
            });
        }
    });
    cmd.try_insert_batch(new_child_of);
    cmd.try_insert_batch(new_base_holder_ref);
    cmd.try_insert_batch(new_cloned_visibility);
}

