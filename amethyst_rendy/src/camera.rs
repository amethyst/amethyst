//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
    math::{Matrix4, },
};
use amethyst_error::Error;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Orthographic {
    matrix: Matrix4<f32>,
}
impl Orthographic {
    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut matrix = Matrix4::<f32>::identity();
        matrix[(0, 0)] = 2.0 / (right - left);
        matrix[(1, 1)] = -2.0 / (top - bottom);
        matrix[(2, 2)] = 1.0 / (z_far - z_near);
        matrix[(0, 3)] = -(right + left) / (right - left);
        matrix[(1, 3)] = -(top + bottom) / (top - bottom);
        matrix[(2, 3)] = -z_near / (z_far - z_near);
        Self { matrix }
    }

    #[inline]
    pub fn top(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn left(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn right(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn near(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn far(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn set_top(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_bottom(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_left(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_right(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn as_matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    #[inline]
    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.matrix
    }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Perspective {
    matrix: Matrix4<f32>,
}
impl Perspective {
    pub fn new(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Self {
        // Important: nalgebra's methods on Perspective3 are not safe for use with RH matrices
        let mut matrix = Matrix4::<f32>::identity();
        let tan_half_fovy = (fov / 2.0).tan();

        matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        matrix[(1, 1)] = -1.0 / tan_half_fovy;
        matrix[(2, 2)] = z_far / (z_far - z_near);
        matrix[(2, 3)] = -(z_near * z_far) / (z_far - z_near);
        matrix[(3, 2)] = 1.0;
        matrix[(3, 3)] = 0.0;

        Self {
            matrix,
        }
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        (self.matrix[(1, 1)] / self.matrix[(0, 0)]).abs()
    }

    #[inline]
    pub fn fovy(&self) -> f32 {
        (-1.0 / self.matrix[(1, 1)]).atan() * 2.0
    }

    #[inline]
    pub fn near(&self) -> f32 {
        1.0 + self.matrix[(2,3)] / self.matrix[(2,2)]
    }

    #[inline]
    pub fn far(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn set_aspect(&mut self, aspect: f32) {
        self.matrix[(0, 0)] = (self.matrix[(1, 1)] / aspect) * -1.0;
    }

    #[inline]
    pub fn set_fov(&mut self, fov: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn as_matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    #[inline]
    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.matrix
    }
}


/// The projection mode of a `Camera`.
///
/// TODO: Remove and integrate with `Camera`.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub enum Projection {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    #[serde(with = "serde_ortho")]
    Orthographic(Orthographic),
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    #[serde(with = "serde_persp")]
    Perspective(Perspective),
  
    /// A raw matrix projection
    Matrix(Matrix4<f32>),
}

impl Projection {
    /// Creates an orthographic projection with the given left, right, bottom, and
    /// top plane distances.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Projection {
        Projection::Orthographic(Orthographic::new(left, right, bottom, top, z_near, z_far))
    }

    /// Creates a perspective projection with the given aspect ratio and
    /// field-of-view. `fov` is specified in radians.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn perspective(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Projection {
        Projection::Perspective(Perspective::new(aspect, fov, z_near, z_far))
    }

    pub fn as_orthographic(&self) -> Result<&Orthographic, failure::Error> {
        match *self {
            Projection::Orthographic(ref s) => Ok(s),
            _ => Err(failure::format_err!("Attempting to retrieve orthographic from invalid projection"))
        }
    }

    pub fn as_perspective(&self) -> Result<&Perspective, failure::Error> {
        match *self {
            Projection::Perspective(ref s) => Ok(s),
            _ => Err(failure::format_err!("Attempting to retrieve perspective from invalid projection")),
        }
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref s) => s.as_matrix(),
            Projection::Perspective(ref s) => s.as_matrix(),
            Projection::Matrix(ref s) => s,
        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref mut s) => s.as_matrix_mut(),
            Projection::Perspective(ref mut s) => s.as_matrix_mut(),
            Projection::Matrix(ref mut s) => s,

        }
    }
}

impl From<Orthographic> for Projection {
    fn from(proj: Orthographic) -> Self {
        Projection::Orthographic(proj)
    }
}

impl From<Perspective> for Projection {
    fn from(proj: Perspective) -> Self {
        Projection::Perspective(proj)
    }
}

impl From<Perspective3<f32>> for Projection {
    fn from(proj: Perspective3<f32>) -> Self {
        // Get fovy, aspect and planes from nalgebra and constrcut new.
        Projection::Perspective(Perspective::new(proj.aspect(), proj.fovy(), proj.znear(), proj.zfar()))
    }
}

impl From<Matrix4<f32>> for Projection {
    fn from(proj: Matrix4<f32>) -> Self {
        Projection::Matrix(proj)
    }
}

impl From<Projection> for Camera {
    fn from(proj: Projection) -> Self {
        Camera { inner: proj }
    }
}

/// Camera struct.
/// 
/// Contains a projection matrix to convert from world/eye-space
/// into normalized device coordinates.
/// For rendy/gfx-hal these are y-down, x-right and y-away in range [0; 1]
/// 
/// World Coordinate system
/// +y
/// |  +z
/// | /
/// |/___+x
/// 
/// NDC system
///  +z
/// /
/// |¯¯¯+x
/// |
/// +y
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub struct Camera {
    /// Graphical projection of the camera.
    inner: Projection,
}

impl Camera {
    /// Create a normalized camera for 2D.
    ///
    /// Will use an orthographic projection centered around (0, 0) of size (width, height)
    /// Bottom left corner is (-width/2.0, -height/2.0)
    /// View transformation will be multiplicative identity.
    pub fn standard_2d(width: f32, height: f32) -> Self {
        // TODO: Check if bottom = height/2.0 is really the solution we want here.
        // Maybe the same problem as with the perspective matrix.
        Self::from(Projection::orthographic(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
            0.1,
            2000.0,
        ))
    }

    /// Create a standard camera for 3D.
    ///
    /// Will use a perspective projection with aspect from the given screen dimensions and a field
    /// of view of π/3 radians (60 degrees).
    /// View transformation will be multiplicative identity.
    pub fn standard_3d(width: f32, height: f32) -> Self {
        Self::from(Projection::perspective(
            width / height,
            std::f32::consts::FRAC_PI_3,
            0.1,
            2000.0,
        ))
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match self.inner {
            Projection::Matrix(ref m) => m,
            Projection::Orthographic(ref p) => p.as_matrix(),
            Projection::Perspective(ref p) => p.as_matrix(),
        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match self.inner {
            Projection::Matrix(ref mut m) => m,
            Projection::Orthographic(ref mut p) => p.as_matrix_mut(),
            Projection::Perspective(ref mut p) => p.as_matrix_mut(),
        }
    }

    pub fn projection(&self) -> &Projection {
        &self.inner
    }

    pub fn set_projection(&mut self, new: Projection) {
        self.inner = new;
    }
}

impl Component for Camera {
    type Storage = HashMapStorage<Self>;
}

/// Active camera resource, used by the renderer to choose which camera to get the view matrix from.
/// If no active camera is found, the first camera will be used as a fallback.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveCamera {
    /// Camera entity
    pub entity: Entity,
}

/// Projection prefab
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CameraPrefab(Projection);

impl<'a> PrefabData<'a> for CameraPrefab {
    type SystemData = WriteStorage<'a, Camera>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage.insert(entity, Camera { inner: self.0.clone() })?;
        Ok(())
    }
}

/// Active camera prefab
pub struct ActiveCameraPrefab(usize);

impl<'a> PrefabData<'a> for ActiveCameraPrefab {
    type SystemData = (Option<Write<'a, ActiveCamera>>,);
    type Result = ();

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        if let Some(ref mut cam) = system_data.0 {
            cam.entity = entities[self.0];
        }
        // TODO: if no `ActiveCamera` insert using `LazyUpdate`, require changes to `specs`
        Ok(())
    }
}

mod serde_ortho {
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<super::Orthographic, D::Error>
        where
            D: Deserializer<'de>,
    {
        let v = Orthographic::deserialize(deserializer)?;
        Ok(super::Orthographic::new(
            v.left, v.right, v.bottom, v.top, v.znear, v.zfar,
        ))
    }

    pub fn serialize<S>(proj: &super::Orthographic, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        Serialize::serialize(
            &Orthographic {
                left: proj.left(),
                right: proj.right(),
                bottom: proj.bottom(),
                top: proj.top(),
                znear: proj.near(),
                zfar: proj.far(),
            },
            serializer,
        )
    }
}

mod serde_persp {
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Perspective {
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<super::Perspective, D::Error>
        where
            D: Deserializer<'de>,
    {
        let v = Perspective::deserialize(deserializer)?;
        Ok(super::Perspective::new(v.aspect, v.fovy, v.znear, v.zfar))
    }

    pub fn serialize<S>(proj: &super::Perspective, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        Serialize::serialize(
            &Perspective {
                aspect: proj.aspect(),
                fovy: proj.fovy(),
                znear: proj.near(),
                zfar: proj.far(),
            },
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::{de::from_str, ser::to_string_pretty};
    use amethyst_core::math::{Point3, Matrix4, Isometry3, Translation3, UnitQuaternion, Vector3, Vector4, convert};
    use amethyst_core::Transform;

    use approx::assert_ulps_eq;
    use more_asserts::{assert_gt, assert_ge, assert_lt, assert_le};

    // TODO: this will be fixed after camera projection refactor
    #[test]
    fn test_orthographic_serde() {
        let test_ortho = Projection::orthographic(0.0, 100.0, 10.0, 150.0, -5.0, 100.0);
        println!("{}", to_string_pretty(&test_ortho, Default::default()).unwrap());

        let de = from_str(&to_string_pretty(&test_ortho, Default::default()).unwrap()).unwrap();
        assert_eq!(test_ortho, de);
    }

    // TODO: this will be fixed after camera projection refactor
    #[test]
    fn test_perspective_serde() {
        let test_persp = Projection::perspective(1.7, std::f32::consts::FRAC_PI_3, 0.1, 1000.0);
        println!("{}", to_string_pretty(&test_persp, Default::default()).unwrap());

        let de = from_str(&to_string_pretty(&test_persp, Default::default()).unwrap()).unwrap();

        assert_eq!(test_persp, de);
    }

    // Our world-space is Y-up, X-right, z-away
    // Thus eye-space is y-up, x-right, z-behind
    // Current render target is y-down, x-right, z-away
    fn setup() -> (Transform, [Point3<f32>; 3], [Point3<f32>; 3]) {
        // Test camera is positioned at (0,0,-3) in world space
        // A camera without rotation points (0,0,-1)
        let camera_transform : Transform = Transform::new(
            Translation3::new(0.0, 0.0, -3.0), 
            UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.0),
            [1.0, 1.0, 1.0].into());

        let simple_points : [Point3<f32>; 3] = [
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0)
        ];

        let simple_points_clipped : [Point3<f32>; 3] = [
            Point3::new(-20.0, 0.0, 0.0),
            Point3::new(0.0, -20.0, 0.0),
            Point3::new(0.0, 0.0, -10.0)
        ];
        (camera_transform, simple_points, simple_points_clipped)
    }

    fn gatherer_calc_view_matrix(transform: Transform) -> Matrix4<f32> {
        convert(transform.view_matrix())
    }

    #[test]
    fn camera_matrix() {
        let (camera_transform, _, _) = setup();
        let iso = Isometry3::face_towards(
            &Point3::new(0.0, 0.0, -3.0), 
            &Point3::new(0.0, 0.0, 0.0), 
            &Vector3::y_axis()
        );
        let our_iso : Matrix4<f32> = convert(camera_transform.isometry().to_homogeneous());
        // Check camera isometry
        assert_ulps_eq!(our_iso, iso.to_homogeneous());

        let view_matrix = Isometry3::look_at_lh(
            &Point3::new(0.0, 0.0, -3.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::y_axis()
        );
        // Check view matrix.
        // The view matrix is used to transfrom a point from world space to eye space.
        // Changes the base of a vector from world origin to your eye.
        let our_view : Matrix4<f32> = convert(camera_transform.view_matrix());
        assert_ulps_eq!(our_view, view_matrix.to_homogeneous(), max_ulps = 10);

        let our_inverse = gatherer_calc_view_matrix(camera_transform);
        assert_ulps_eq!(our_inverse, view_matrix.to_homogeneous());

        assert_ulps_eq!(our_inverse, our_view);
    }

    #[test]
    fn standard_2d() {
        let width = 1280.0;
        let height = 720.0;
        let top = height/2.0;
        let bottom = -height/2.0;
        let left = -width/2.0;
        let right = width/2.0;

        // Our standrd projection has a far clipping plane of 2000.0
        let proj = Projection::orthographic(left, right, bottom, top, 0.1, 2000.0);
        let our_proj = Camera::standard_2d(width, height).inner;

        assert_ulps_eq!(our_proj.as_matrix(), proj.as_matrix());
    }

    #[test]
    fn standard_3d() {
        let width = 1280.0;
        let height = 720.0;

        // Our standrd projection has a far clipping plane of 2000.0
        let proj = Projection::perspective(width/height, std::f32::consts::FRAC_PI_3, 0.1, 2000.0);
        let our_proj = Camera::standard_3d(width, height).inner;

        assert_ulps_eq!(our_proj.as_matrix(), proj.as_matrix());
    }


    #[test]
    fn perspective_orientation() {
        let (camera_transform, simple_points, simple_points_clipped) = setup();

        let proj = Projection::perspective(1280.0/720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let x_axis = mvp * simple_points[0].to_homogeneous();
        let y_axis = mvp * simple_points[1].to_homogeneous();
        let z_axis = mvp * simple_points[2].to_homogeneous();
      
        assert_gt!(x_axis[0], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);

        // Z should be in [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_le!(z_axis[2], 1.0);
        // Should be near the near plane at 0.0
        assert_le!(z_axis[2], 0.5);

        let x_axis_clipped = mvp * simple_points_clipped[0].to_homogeneous();
        let y_axis_clipped = mvp * simple_points_clipped[1].to_homogeneous();
        let z_axis_clipped = mvp * simple_points_clipped[2].to_homogeneous();

        // Outside of frustum should be clipped
        assert_le!(x_axis_clipped[0], -1.0);
        assert_ge!(y_axis_clipped[1], 1.0);

        // Behind Camera should be clipped.
        assert_lt!(z_axis_clipped[2], 0.0);
    }

    // Todo: Add perspective_orientation_reversed_z when we support reversed z depth buffer.
  
  
    #[test]
    fn orthographic_orientation() {
        let (camera_transform, simple_points, _) = setup();

        let proj = Projection::orthographic(-1280.0/2.0, 1280.0/2.0, -720.0/2.0, 720.0/2.0, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let x_axis = mvp * simple_points[0].to_homogeneous();
        let y_axis = mvp * simple_points[1].to_homogeneous();
        let z_axis = mvp * simple_points[2].to_homogeneous();

        assert_gt!(x_axis[0], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);

        // Z should be in [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_le!(z_axis[2], 1.0);
    }

    #[test]
    fn perspective_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Projection::perspective(1280.0/720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;
        // Nearest point = -distance to (0,0) + zNear
        let near = Point3::new(0.0, 0.0, -2.9);
      
        assert_ulps_eq!((mvp * near.to_homogeneous())[2], 0.0);

        // Furthest point = -distance to (0,0) + zFar
        let far = Point3::new(0.0, 0.0, 97.0);
        assert_ulps_eq!((mvp * far.to_homogeneous())[2], 1.0);
    }

    #[test]
    fn orthographic_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Projection::orthographic(-1280.0/2.0, 1280.0/2.0, -720.0/2.0, 720.0/2.0, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;
        // Nearest point = -distance to (0,0) + zNear
        let near = Point3::new(0.0, 0.0, -2.9);
        assert_ulps_eq!((mvp * near.to_homogeneous())[2], 0.0);

        // Furthest point = -distance to (0,0) + zFar
        let far = Point3::new(0.0, 0.0, 97.0);
        assert_ulps_eq!((mvp * far.to_homogeneous())[2], 1.0);
    }

    #[test]
    fn perspective_project_cube_centered() {
        let (camera_transform, _, _) = setup();

        // Cube in worldspace
        let cube = [
            Point3::new(1.0, 1.0, 1.0), Point3::new(-1.0, 1.0, 1.0), 
            Point3::new(-1.0, -1.0, 1.0), Point3::new(1.0, -1.0, 1.0),

            Point3::new(1.0, 1.0, -1.0), Point3::new(-1.0, 1.0, -1.0), 
            Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, -1.0, -1.0),
        ];

        let proj = Projection::perspective(1280.0/720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let _result : Vec<Vector4<f32>> = cube.into_iter().map(|vertex| mvp * vertex.to_homogeneous()).collect();
        // TODO: Calc correct result
        // assert_ulps_eq!(result, Point());
        unimplemented!()
    }

    #[test]
    fn perspective_project_cube_off_centered_rotated() {
        let (camera_transform, _, _) = setup();
        // Cube in worldspace
        let cube = [
            Point3::new(1.0, 1.0, 1.0), Point3::new(-1.0, 1.0, 1.0), 
            Point3::new(-1.0, -1.0, 1.0), Point3::new(1.0, -1.0, 1.0),

            Point3::new(1.0, 1.0, -1.0), Point3::new(-1.0, 1.0, -1.0), 
            Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, -1.0, -1.0),
        ];
        let proj = Projection::perspective(1280.0/720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        // Rotated x and y axis by 45°
        let rotation = UnitQuaternion::from_euler_angles(std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4, 0.0);
        let model = Isometry3::from_parts(Translation3::new(-1.0, 0.0, 0.0), rotation).to_homogeneous();

        let mvp = proj.as_matrix() * view * model;

        // Todo: Maybe check more cells of the model matrix
        assert_ulps_eq!(model.column(0)[0], 0.70710678118);
        assert_ulps_eq!(model.column(3)[0], -1.0);

        let result : Vec<Vector4<f32>> = cube.iter().map(|vertex| mvp * vertex.to_homogeneous()).collect();

        // TODO: Calc correct result
        // assert_ulps_eq!(result, Point());
        unimplemented!()
    }
  
    #[test]
    fn orthographic_project_cube_off_centered_rotated() {
        unimplemented!()
    }
}

