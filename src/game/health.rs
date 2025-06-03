#![allow(unreachable_code)]

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use super::status_bar::{GetValue, IntoStatusBar};

#[auto_register_type]
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Health(pub f32);

#[auto_register_type]
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Health = enforce_exists!(Health))]
pub struct MaxHealth(pub f32);

#[derive(Event, Debug)]
pub struct AdjustHp {
    pub target: Entity,
    pub amount: f32,
}

#[auto_register_type]
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Dead;

// Plugin
#[auto_plugin(app=app)]
pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_adjust_hp);
    app.add_event::<AdjustHp>();
}

// Internals

fn handle_adjust_hp(
    mut commands: Commands,
    mut damage_reader: EventReader<AdjustHp>,
    mut health_query: Query<&mut Health, Without<Dead>>,
) {
    for AdjustHp { target, amount } in damage_reader.read() {
        let Ok(mut health) = health_query.get_mut(*target) else {
            continue;
        };
        health.0 += amount;

        if health.0 <= 0.0 {
            commands.entity(*target).insert(Dead);
        }
    }
}

impl AdjustHp {
    pub fn new(target: Entity, amount: f32) -> Self {
        Self { target, amount }
    }
}

impl GetValue for Health {
    fn get(&self) -> f32 {
        self.0
    }
}

impl GetValue for MaxHealth {
    fn get(&self) -> f32 {
        self.0
    }
}

impl IntoStatusBar for Health {
    type GetValue = Health;
    type GetMaxValue = MaxHealth;
}
