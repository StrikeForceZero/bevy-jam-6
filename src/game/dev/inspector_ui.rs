// Derived from: https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/crates/bevy-inspector-egui/examples/integrations/egui_dock.rs
// Original License:
//      MIT - https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/LICENSE-MIT.md
//  or
//      Apache 2.0 - https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/LICENSE-APACHE.md

// Derived from: https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/examples/bevy/src/gui.rs
// Original License:
//      MIT - https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/LICENSE-APACHE
//  or
//      Apache 2.0 - https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/LICENSE-MIT

use crate::game::camera::MainCamera;
use crate::game::dev::selection::{DebugSelect, DebugSelectEnabled, DebugSelected};
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::log::tracing_subscriber::field::debug;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::render::camera::Viewport;
use bevy::window::PrimaryWindow;
use bevy_auto_plugin::auto_plugin::*;
use bevy_egui::{EguiContext, EguiContextSettings, EguiPostUpdateSet};
#[cfg(feature = "egui_inspector")]
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_inspector::{
        self, EntityFilter, Filter, guess_entity_name,
        hierarchy::{Hierarchy, SelectedEntities, SelectionMode},
        ui_for_entities_shared_components, ui_for_entity_with_children,
    },
    egui,
};
#[cfg(feature = "egui_dock")]
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::any::TypeId;
use std::fmt::Debug;
use std::hash::Hash;

use super::DebugUiEnabled;

#[auto_plugin(app=app)]
pub(super) fn plugin(app: &mut App) {
    #[cfg(feature = "egui_inspector")]
    app.add_plugins(DefaultInspectorConfigPlugin);
    app.insert_resource(UiState::new());
    app.add_event::<DebugSelect>();
    app.add_systems(
        PreUpdate,
        disable_bevy_picking.after(bevy_egui::EguiPreUpdateSet::InitContexts),
    );
    app.add_systems(
        PostUpdate,
        show_ui_system
            .before(EguiPostUpdateSet::ProcessOutput)
            .before(bevy_egui::end_pass_system)
            .before(TransformSystem::TransformPropagate),
    );
    app.add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
    // app.add_systems(
    //     PreUpdate,
    //     absorb_egui_inputs
    //         .after(bevy_egui::EguiPreUpdateSet::ProcessInput)
    //         .before(bevy_egui::EguiPreUpdateSet::BeginPass),
    // );
    app.register_type::<Option<Handle<Image>>>();
    app.register_type::<AlphaMode>();
}

struct NoObserverFilter {
    filter: Filter,
    show_observers: bool,
}

impl NoObserverFilter {
    fn from_ui(ui: &mut egui::Ui) -> Self {
        let id = egui::Id::new("no_observer_filter");
        let filter = Filter::from_ui(ui, id);
        let show_observers = {
            let id = id.with("show_observers");
            let mut show_observers = ui.memory_mut(|mem| {
                let show_observers: &mut bool = mem.data.get_persisted_mut_or_default(id);
                *show_observers
            });
            ui.checkbox(&mut show_observers, "Show Observers");
            ui.memory_mut(|mem| {
                *mem.data.get_persisted_mut_or_default(id) = show_observers;
            });
            show_observers
        };
        Self {
            filter,
            show_observers,
        }
    }
}
impl EntityFilter for NoObserverFilter {
    type StaticFilter = ();

    fn filter_entity(&self, world: &mut World, entity: Entity) -> bool {
        fn is_observer(name: &str) -> bool {
            let name = name.to_lowercase();
            name.starts_with("observer ")
        }
        self.filter.filter_entity(world, entity) && {
            let name = guess_entity_name(world, entity);
            let is_observer = is_observer(&name);
            let hide_observers = !self.show_observers;
            if is_observer
                && hide_observers
                && !self.filter.word.to_lowercase().starts_with("observer")
            {
                return false;
            }
            true
        }
    }
}

fn hierarchy_ui_with_filter(
    world: &mut World,
    ui: &mut egui::Ui,
    selected: &mut SelectedEntities,
    filter: impl EntityFilter,
) -> bool {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    Hierarchy {
        world,
        type_registry: &type_registry,
        selected,
        context_menu: None,
        shortcircuit_entity: None,
        extra_state: &mut (),
    }
    .show_with_filter::<Without<ChildOf>, _>(ui, filter)
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Option<Single<Mut<Window>, With<PrimaryWindow>>>,
    egui_settings: Single<&EguiContextSettings>,
    main_camera: Option<Single<Mut<Camera>, With<MainCamera>>>,
) {
    let Some(mut cam) = main_camera else {
        return;
    };

    let Some(window) = primary_window else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
pub(super) struct UiState {
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    last_selection_action: HashMap<EguiWindow, Option<(SelectionMode, Entity)>>,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            last_selection_action: Default::default(),
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            last_selection_action: &mut self.last_selection_action,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }

    pub fn selected_entities_mut(&mut self) -> &mut SelectedEntities {
        &mut self.selected_entities
    }

    pub fn selected_entities(&self) -> &SelectedEntities {
        &self.selected_entities
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    last_selection_action: &'a mut HashMap<EguiWindow, Option<(SelectionMode, Entity)>>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
                if self
                    .world
                    .get_resource::<DebugUiEnabled>()
                    .is_none_or(|enabled| !enabled.0)
                {
                    return;
                }
                egui::Frame::new()
                    .outer_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        let mut debug_select_enabled =
                            self.world.resource::<DebugSelectEnabled>().0;
                        ui.checkbox(&mut debug_select_enabled, "Enable Debug Select");
                        if debug_select_enabled != self.world.resource::<DebugSelectEnabled>().0 {
                            self.world.resource_mut::<DebugSelectEnabled>().0 =
                                debug_select_enabled;
                        }
                    });
            }
            EguiWindow::Hierarchy => {
                let filter = NoObserverFilter::from_ui(ui);
                let selected =
                    hierarchy_ui_with_filter(self.world, ui, self.selected_entities, filter);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => {
                {
                    let last_selection_action =
                        self.last_selection_action.entry(*window).or_default();
                    let last_action = self.selected_entities.last_action();
                    let last_action_changed = {
                        type SM = SelectionMode;
                        // SelectionMode doesn't impl PartialEq
                        match (*last_selection_action, last_action) {
                            (None, None) => false,
                            (Some((SM::Replace, a)), Some((SM::Replace, b))) => a != b,
                            (Some((SM::Extend, a)), Some((SM::Extend, b))) => a != b,
                            (Some((SM::Add, a)), Some((SM::Add, b))) => a != b,
                            _ => true,
                        }
                    };
                    if last_action_changed {
                        *last_selection_action = last_action;
                        log::debug!("inspector entity selection changed {last_action:?}");
                        if let Some((selection_mode, selection_action_entity)) = last_action {
                            let mut selected_entities_to_remove = self
                                .world
                                .query_filtered::<Entity, With<DebugSelected>>()
                                .iter(self.world)
                                .collect::<HashSet<_>>();
                            for current_entity in self.selected_entities.iter() {
                                let Some(has_selected) = self
                                    .world
                                    .query::<Has<DebugSelected>>()
                                    .get(self.world, current_entity)
                                    .ok()
                                else {
                                    continue;
                                };
                                let mut commands = self.world.commands();
                                if let SelectionMode::Replace = selection_mode {
                                    // clear any not in current set
                                    if current_entity != selection_action_entity {
                                        commands.send_event::<DebugSelect>(DebugSelect::Remove(
                                            current_entity,
                                        ));
                                        continue;
                                    }
                                };
                                selected_entities_to_remove.remove(&current_entity);
                                // add components if needed
                                if !has_selected {
                                    commands.send_event::<DebugSelect>(DebugSelect::Extend(
                                        current_entity,
                                    ));
                                }
                            }
                            for entity in selected_entities_to_remove {
                                self.world
                                    .send_event::<DebugSelect>(DebugSelect::Remove(entity));
                            }
                        } else {
                            // clear all
                            self.world.send_event::<DebugSelect>(DebugSelect::Clear);
                        }
                    }
                }
                match *self.selection {
                    InspectorSelection::Entities => match self.selected_entities.as_slice() {
                        &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                        entities => ui_for_entities_shared_components(self.world, entities, ui),
                    },
                    InspectorSelection::Resource(type_id, ref name) => {
                        ui.label(name);
                        bevy_inspector::by_type_id::ui_for_resource(
                            self.world,
                            type_id,
                            ui,
                            name,
                            &type_registry,
                        )
                    }
                    InspectorSelection::Asset(type_id, ref name, handle) => {
                        ui.label(name);
                        bevy_inspector::by_type_id::ui_for_asset(
                            self.world,
                            type_id,
                            handle,
                            ui,
                            &type_registry,
                        );
                    }
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

// TODO: this works but requires modification by checking if inside ui_state.viewport or not
//  since the  game view is considered part of the egui window
#[allow(unused)]
fn absorb_egui_inputs(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut contexts: bevy_egui::EguiContexts,
) {
    let ctx = contexts.ctx_mut();

    // Check if egui is interacting with the pointer or keyboard
    if ctx.is_pointer_over_area() || ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
        // Reset all mouse button inputs
        mouse.reset_all();

        // Reset all keyboard inputs
        let pressed_keys: Vec<KeyCode> = keyboard.get_pressed().cloned().collect();
        keyboard.reset_all();

        // Optionally re-apply modifier keys (e.g., Shift, Ctrl) if needed
        for key in pressed_keys {
            if matches!(
                key,
                KeyCode::ShiftLeft | KeyCode::ControlLeft | KeyCode::AltLeft | KeyCode::SuperLeft
            ) {
                keyboard.press(key);
            }
        }
    }
}

fn disable_bevy_picking(
    mut contexts: Query<Mut<EguiContextSettings>, (With<PrimaryWindow>, With<EguiContext>)>,
) {
    for mut context_settings in contexts.iter_mut() {
        if context_settings.capture_pointer_input {
            log::warn!("Disabling bevy_picking input for egui");
            context_settings.capture_pointer_input = false;
        }
    }
}
