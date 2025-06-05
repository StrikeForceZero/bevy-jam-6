use crate::game::utils::extensions::vec2::Vec2Ext;
use crate::game::utils::extensions::vec3::Vec3Ext;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Copy, Clone, Reflect)]
pub enum Vector {
    /// This Vec2’s `.x` = X₃D, `.y` = Z₃D; implicit Y₃D = 0.0.
    XZ_3D(Vec2),

    /// This Vec2’s `.x` = Y₃D, `.y` = Z₃D; implicit X₃D = 0.0.
    YZ_3D(Vec2),

    /// This Vec2 is a screen‐space (X₂D, Y₂D).
    /// When converting to 3D it's treated as Y₂D → −Z₃D so that
    /// “up in 2D” becomes “forward in −Z.”
    XY_2D(Vec2),

    /// This Vec2 is really (X₃D, Y₃D); implicit Z₃D = 0.0.
    XY_3D(Vec2),

    /// A full 3–component vector (X₃D, Y₃D, Z₃D).
    XYZ_3D(Vec3),
}

impl Vector {
    pub fn to_2d(self) -> Vec2 {
        match self {
            Self::XY_2D(vec2) => vec2,
            Self::XZ_3D(vec2) => vec2,
            Self::YZ_3D(Vec2 { y: z, .. }) => Vec2::new(0.0, -z),
            Self::XY_3D(Vec2 { x, .. }) => Vec2::new(x, 0.0),
            Self::XYZ_3D(vec3) => vec3.to_vec2(),
        }
    }
    pub fn to_3d(self) -> Vec3 {
        match self {
            Self::XY_2D(vec2) => vec2.to_vec3(),
            Self::XZ_3D(Vec2 { x, y: z }) => Vec3::new(x, 0.0, z),
            Self::YZ_3D(Vec2 { x: y, y: z }) => Vec3::new(0.0, y, z),
            Self::XY_3D(vec2) => vec2.extend(0.0),
            Self::XYZ_3D(vec3) => vec3,
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use Vector::*;
    const EMPTY: f32 = 0.0;
    const X: f32 = 1.0;
    const Y: f32 = 2.0;
    const Z: f32 = 3.0;

    fn vec2(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }

    #[test]
    fn to_2d() {
        assert_eq!(XY_2D(vec2(X, Y)).to_2d(), vec2(X, Y));
        assert_eq!(XZ_3D(vec2(X, Z)).to_2d(), vec2(X, Z));
        assert_eq!(YZ_3D(vec2(Y, Z)).to_2d(), vec2(EMPTY, -Z));
        assert_eq!(XY_3D(vec2(X, Y)).to_2d(), vec2(X, EMPTY));
        assert_eq!(XYZ_3D(vec3(X, Y, Z)).to_2d(), vec2(X, -Z));
    }

    #[test]
    fn to_3d() {
        assert_eq!(XY_2D(vec2(X, Y)).to_3d(), vec3(X, EMPTY, -Y));
        assert_eq!(XZ_3D(vec2(X, Z)).to_3d(), vec3(X, EMPTY, Z));
        assert_eq!(XY_3D(vec2(X, Y)).to_3d(), vec3(X, Y, EMPTY));
        assert_eq!(XYZ_3D(vec3(X, Y, Z)).to_3d(), vec3(X, Y, Z));
    }
}
