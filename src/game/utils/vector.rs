use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Reflect)]
pub struct XY_2D(pub Vec2);

impl XY_2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
    const fn inner(self) -> Vec2 {
        self.0
    }
    pub const fn x(self) -> f32 {
        self.inner().x
    }
    pub const fn y(self) -> f32 {
        self.inner().y
    }
    pub const fn to_vector(self) -> Vector {
        Vector::XY_2D(self)
    }
}

impl From<XY_2D> for Vector {
    fn from(xy_2d: XY_2D) -> Self {
        Self::XY_2D(xy_2d)
    }
}

impl From<Vec2> for XY_2D {
    fn from(vec2: Vec2) -> Self {
        Self(vec2)
    }
}

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect)]
pub struct XZ_3D(pub Vec2);

impl XZ_3D {
    pub const fn new(x: f32, z: f32) -> Self {
        Self(Vec2::new(x, z))
    }
    pub const fn x(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().x
    }
    pub const fn z(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().z
    }
    pub const fn to_vector(self) -> Vector {
        Vector::XZ_3D(self)
    }
}

impl From<XZ_3D> for Vector {
    fn from(xz_3d: XZ_3D) -> Self {
        Self::XZ_3D(xz_3d)
    }
}

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect)]
pub struct YZ_3D(pub Vec2);

impl YZ_3D {
    pub const fn new(y: f32, z: f32) -> Self {
        Self(Vec2::new(y, z))
    }
    pub const fn y(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().y
    }
    pub const fn z(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().z
    }
    pub const fn to_vector(self) -> Vector {
        Vector::YZ_3D(self)
    }
}

impl From<YZ_3D> for Vector {
    fn from(yz_3d: YZ_3D) -> Self {
        Self::YZ_3D(yz_3d)
    }
}

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect)]
pub struct XY_3D(pub Vec2);

impl XY_3D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
    pub const fn x(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().x
    }
    pub const fn y(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().y
    }
    pub const fn to_vector(self) -> Vector {
        Vector::XY_3D(self)
    }
}

impl From<XY_3D> for Vector {
    fn from(xy_3d: XY_3D) -> Self {
        Self::XY_3D(xy_3d)
    }
}

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Default, Copy, Clone, PartialEq, Reflect)]
pub struct XYZ_3D(pub Vec3);

impl XYZ_3D {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
    const fn inner(self) -> Vec3 {
        self.0
    }
    pub const fn x(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().x
    }
    pub const fn y(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().y
    }
    pub const fn z(self) -> f32 {
        self.to_vector().to_bevy_3d().inner().z
    }
    pub const fn xy(self) -> XY_3D {
        XY_3D::new(self.x(), self.y())
    }
    pub const fn yz(self) -> YZ_3D {
        YZ_3D::new(self.y(), self.z())
    }
    pub const fn xz(self) -> XZ_3D {
        XZ_3D::new(self.x(), self.z())
    }
    pub const fn to_vector(self) -> Vector {
        Vector::XYZ_3D(self)
    }
}

impl From<XYZ_3D> for Vector {
    fn from(xyz_3d: XYZ_3D) -> Self {
        Self::XYZ_3D(xyz_3d)
    }
}

impl From<Vec3> for XYZ_3D {
    fn from(vec3: Vec3) -> Self {
        Self(vec3)
    }
}

pub const fn to_bevy_2d(vec3: Vec3) -> Vec2 {
    let Vec3 { x, y, z } = vec3;
    Vec2::new(x, -z)
}

pub const fn to_bevy_3d(vec2: Vec2) -> Vec3 {
    let Vec2 { x, y: z } = vec2;
    Vec3::new(x, 0.0, -z)
}

#[allow(non_camel_case_types)]
#[auto_register_type]
#[derive(Debug, Copy, Clone, PartialEq, Reflect)]
pub enum Vector {
    /// This Vec2’s `.x` = X₃D, `.y` = Z₃D; implicit Y₃D = 0.0.
    XZ_3D(XZ_3D),

    /// This Vec2’s `.x` = Y₃D, `.y` = Z₃D; implicit X₃D = 0.0.
    YZ_3D(YZ_3D),

    /// This Vec2 is a screen‐space (X₂D, Y₂D).
    /// When converting to 3D it's treated as Y₂D → −Z₃D so that
    /// “up in 2D” becomes “forward in −Z.”
    XY_2D(XY_2D),

    /// This Vec2 is really (X₃D, Y₃D); implicit Z₃D = 0.0.
    XY_3D(XY_3D),

    /// A full 3–component vector (X₃D, Y₃D, Z₃D).
    XYZ_3D(XYZ_3D),
}

impl Vector {
    pub const fn to_bevy_2d(self) -> XY_2D {
        XY_2D(match self {
            Self::XY_2D(XY_2D(vec2)) => vec2,
            Self::XZ_3D(XZ_3D(vec2)) => vec2,
            Self::YZ_3D(YZ_3D(Vec2 { y: z, .. })) => Vec2::new(0.0, -z),
            Self::XY_3D(XY_3D(Vec2 { x, .. })) => Vec2::new(x, 0.0),
            Self::XYZ_3D(XYZ_3D(vec3)) => to_bevy_2d(vec3),
        })
    }
    pub const fn to_bevy_3d(self) -> XYZ_3D {
        XYZ_3D(match self {
            Self::XY_2D(XY_2D(vec2)) => to_bevy_3d(vec2),
            Self::XZ_3D(XZ_3D(Vec2 { x, y: z })) => Vec3::new(x, 0.0, z),
            Self::YZ_3D(YZ_3D(Vec2 { x: y, y: z })) => Vec3::new(0.0, y, z),
            Self::XY_3D(XY_3D(vec2)) => vec2.extend(0.0),
            Self::XYZ_3D(XYZ_3D(vec3)) => vec3,
        })
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    const EMPTY: f32 = 0.0;
    const X: f32 = 1.0;
    const Y: f32 = 2.0;
    const Z: f32 = 3.0;

    fn d2(x: f32, y: f32) -> XY_2D {
        XY_2D::new(x, y)
    }

    fn d3(x: f32, y: f32, z: f32) -> XYZ_3D {
        XYZ_3D::new(x, y, z)
    }

    fn vec2(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }

    #[test]
    fn to_bevy_2d() {
        assert_eq!(XY_2D::new(X, Y).to_vector().to_bevy_2d(), d2(X, Y));
        assert_eq!(XZ_3D::new(X, Z).to_vector().to_bevy_2d(), d2(X, Z));
        assert_eq!(YZ_3D::new(Y, Z).to_vector().to_bevy_2d(), d2(EMPTY, -Z));
        assert_eq!(XY_3D::new(X, Y).to_vector().to_bevy_2d(), d2(X, EMPTY));
        assert_eq!(XYZ_3D::new(X, Y, Z).to_vector().to_bevy_2d(), d2(X, -Z));
    }

    #[test]
    fn to_bevy_3d() {
        assert_eq!(XY_2D::new(X, Y).to_vector().to_bevy_3d(), d3(X, EMPTY, -Y));
        assert_eq!(XZ_3D::new(X, Z).to_vector().to_bevy_3d(), d3(X, EMPTY, Z));
        assert_eq!(XY_3D::new(X, Y).to_vector().to_bevy_3d(), d3(X, Y, EMPTY));
        assert_eq!(XYZ_3D::new(X, Y, Z).to_vector().to_bevy_3d(), d3(X, Y, Z));
    }
}
