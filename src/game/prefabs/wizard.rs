use crate::game::asset_tracking::LoadResource;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Resource, Asset, Debug, Clone, Reflect)]
pub struct WizardAssets {
    #[dependency]
    pub wizard: Handle<Scene>,
}

impl FromWorld for WizardAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            wizard: assets.load(GltfAssetLabel::Scene(0).from_asset("models/wizard/wizard.glb")),
        }
    }
}

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Wizard;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<WizardAssets>();
    app.add_observer(on_wizard_added);
}

fn on_wizard_added(
    trigger: Trigger<OnAdd, Wizard>,
    wizard: Res<WizardAssets>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.target())
        .insert(SceneRoot(wizard.wizard.clone()));
}
