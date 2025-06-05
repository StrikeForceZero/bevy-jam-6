use crate::game::utils::vector::to_bevy_2d;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

pub trait Vec3Ext {
    // TODO: rename to to_bevy_2d
    fn to_vec2(self) -> Vec2;
}

impl Vec3Ext for Vec3 {
    fn to_vec2(self) -> Vec2 {
        to_bevy_2d(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vec3_to_vec2() {
        assert_eq!(Vec3::new(1.0, 2.0, 3.0).to_vec2(), Vec2::new(1.0, -3.0));
    }
}
