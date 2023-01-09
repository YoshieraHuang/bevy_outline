use std::ops::Deref;

use bevy::{
    app::PluginGroupBuilder,
    ecs::{schedule::ShouldRun, system::EntityCommands},
    prelude::{
        App, Changed, Commands, CoreStage, Deref, Entity, Handle, Or, Plugin, PluginGroup, Query,
        Res, SystemSet,
    },
    ui::Interaction,
};

use bevy_mod_picking::{
    InteractablePickingPlugin, PausedForBlockers, PickingPlugin, PickingPluginsState,
    PickingSystem, Selection,
};

use crate::{OutlineMaterial, OutlinePlugin};

/// Alternative to the `bevy_mod_picking`'s `DefaultPickingPlugins`.
/// Object get outlined instead of changing materials when hovered, clicked or selected.
pub struct DefaultPickingPlugins;
impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PickingPlugin);
        group.add(InteractablePickingPlugin);
        group.add(OutlinePlugin);
        group.add(OutlinePickingPlugin);
    }
}

/// `OutlineMaterial` handle resource used when object is hovered.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref)]
pub struct HoverOutline(pub Handle<OutlineMaterial>);

/// `OutlineMaterial` handle resource used when object is selected.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref)]
pub struct SelectedOutline(pub Handle<OutlineMaterial>);

/// `OutlineMaterial` handle resource used when object is pressed or clicked.
/// If this resource does not exist in world, no outline will show.
#[derive(Deref)]
pub struct PressedOutline(pub Handle<OutlineMaterial>);

/// Outline picking plugin as an alternative to `HighlightablePickingPlugin` in `bevy_mod_picking`
pub struct OutlinePickingPlugin;

impl Plugin for OutlinePickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_run_criteria(|state: Res<PickingPluginsState>| {
                    if state.enable_highlighting {
                        ShouldRun::Yes
                    } else {
                        ShouldRun::No
                    }
                })
                .with_system(
                    mesh_highlighting
                        .label(PickingSystem::Highlighting)
                        .before(PickingSystem::Events),
                ),
        );
    }
}

/// Similiar to the `mesh_highlighting` system in `bevy_mod_picking`
#[allow(clippy::type_complexity)]
fn mesh_highlighting(
    paused: Option<Res<PausedForBlockers>>,
    mut commands: Commands,
    hover_outline: Option<Res<HoverOutline>>,
    pressed_outline: Option<Res<PressedOutline>>,
    selected_outline: Option<Res<SelectedOutline>>,
    mut interaction_query: Query<
        (Entity, &Interaction, Option<&Selection>),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (ent, _, selection) in interaction_query.iter_mut() {
                if let Some(selection) = selection {
                    let mut entity_commands = commands.entity(ent);
                    if selection.selected() {
                        set_outline(&mut entity_commands, &selected_outline);
                        continue;
                    }
                    entity_commands.remove::<Handle<OutlineMaterial>>();
                }
            }
            return;
        }
    }
    for (ent, interaction, selection) in interaction_query.iter_mut() {
        let mut entity_commands = commands.entity(ent);
        match *interaction {
            Interaction::Clicked => {
                set_outline(&mut entity_commands, &pressed_outline);
            }
            Interaction::Hovered => {
                set_outline(&mut entity_commands, &hover_outline);
            }
            Interaction::None => {
                if let Some(selection) = selection {
                    if selection.selected() {
                        set_outline(&mut entity_commands, &selected_outline);
                        continue;
                    }
                }
                entity_commands.remove::<Handle<OutlineMaterial>>();
            }
        };
    }
}

#[inline]
fn set_outline<T: Deref<Target = Handle<OutlineMaterial>> + Send + Sync + 'static>(
    entity_commands: &mut EntityCommands,
    outline: &Option<Res<T>>,
) {
    if let Some(outline) = outline.as_ref() {
        entity_commands.insert((*outline).clone());
    } else {
        entity_commands.remove::<Handle<OutlineMaterial>>();
    }
}
