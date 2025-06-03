use bevy::{
    color::palettes::css::{GRAY, RED},
    prelude::*,
    sprite::Anchor,
};

use crate::game::{
    health::Health,
    status_bar::{StatusBarOf, StatusBarType},
};

// TODO diversify

pub fn overhead_health_bar(target: Entity, y_offset: f32) -> impl Bundle {
    (
        Transform::from_xyz(0.0, y_offset, 0.0),
        InheritedVisibility::default(),
        children![
            (
                StatusBarOf(target),
                StatusBarType::<Health>::default(),
                Sprite {
                    color: RED.into(),
                    custom_size: Some(Vec2::new(145.0, 35.0)),
                    anchor: Anchor::CenterLeft,
                    ..default()
                }
            ),
            (
                Transform::from_xyz(0.0, 0.0, -1.0),
                Sprite {
                    color: GRAY.into(),
                    custom_size: Some(Vec2::new(145.0, 35.0)),
                    anchor: Anchor::CenterLeft,
                    ..default()
                }
            )
        ],
    )
}
