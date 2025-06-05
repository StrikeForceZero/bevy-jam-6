use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

pub trait Vec2Ext {
    // TODO: rename to to_bevy_3d
    fn to_vec3(self) -> Vec3;
}

impl Vec2Ext for Vec2 {
    fn to_vec3(self) -> Vec3 {
        let Vec2 { x, y: z } = self;
        Vec3::new(x, 0.0, -z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vec2_to_vec3() {
        assert_eq!(Vec2::new(1.0, 2.0).to_vec3(), Vec3::new(1.0, 0.0, -2.0));
    }
}
