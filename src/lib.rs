mod apply_mutations;
pub mod colors;
mod deferred_system;
mod elements;
#[macro_use]
mod events;
mod hooks;
#[cfg(feature = "hot_reload")]
mod hot_reload;
mod parse_attributes;
mod tick;

use self::{
    apply_mutations::BevyTemplate, deferred_system::DeferredSystemRegistry, events::EventReaders,
    hooks::EcsSubscriptions, tick::tick_dioxus_ui,
};
use bevy::{
    app::{App, Plugin, Update},
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    prelude::Deref,
    ui::node_bundles::NodeBundle,
    utils::{EntityHashMap, HashMap},
};
use dioxus::core::{Element, ElementId, Scope, VirtualDom};

pub mod prelude {
    pub use super::elements::*;
    pub use super::hooks::*;
    pub use super::{DioxusUiBundle, DioxusUiPlugin, DioxusUiRoot};
    pub use dioxus;
    pub use dioxus::prelude::*;
}

pub struct DioxusUiPlugin;

impl Plugin for DioxusUiPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "hot_reload")]
        dioxus_hot_reload::hot_reload_init!(dioxus_hot_reload::Config::<
            hot_reload::HotReloadContext,
        >::default());

        app.init_non_send_resource::<UiContext>()
            .init_resource::<DeferredSystemRegistry>()
            .init_resource::<EventReaders>()
            .add_systems(Update, tick_dioxus_ui);
    }
}

#[derive(Bundle)]
pub struct DioxusUiBundle {
    pub dioxus_ui_root: DioxusUiRoot,
    pub node_bundle: NodeBundle,
}

#[derive(Component, Deref, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DioxusUiRoot(pub fn(Scope) -> Element);

#[derive(Default)]
struct UiContext {
    roots: HashMap<(Entity, DioxusUiRoot), UiRoot>,
    subscriptions: EcsSubscriptions,
}

struct UiRoot {
    virtual_dom: VirtualDom,
    element_id_to_bevy_ui_entity: HashMap<ElementId, Entity>,
    bevy_ui_entity_to_element_id: EntityHashMap<Entity, ElementId>,
    templates: HashMap<String, BevyTemplate>,
    needs_rebuild: bool,
}

impl UiRoot {
    fn new(root_component: DioxusUiRoot) -> Self {
        Self {
            virtual_dom: VirtualDom::new(root_component.0),
            element_id_to_bevy_ui_entity: HashMap::new(),
            bevy_ui_entity_to_element_id: EntityHashMap::default(),
            templates: HashMap::new(),
            needs_rebuild: true,
        }
    }
}
