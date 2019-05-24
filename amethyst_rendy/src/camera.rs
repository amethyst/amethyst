//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
    math::Matrix4,
};

use amethyst_error::Error;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Orthographic {
    matrix: Matrix4<f32>,
}
impl Orthographic {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Self {
        if cfg!(debug_assertions) {
            assert!(
                !approx::relative_eq!(z_far - z_near, 0.0),
                "The near-plane and far-plane must not be superimposed."
            );
        }

        let mut matrix = Matrix4::<f32>::identity();

        matrix[(0, 0)] = 2.0 / (right - left);
        matrix[(1, 1)] = -2.0 / (top - bottom);
        matrix[(2, 2)] = -1.0 / (z_far - z_near);
        matrix[(0, 3)] = -(right + left) / (right - left);
        matrix[(1, 3)] = -(top + bottom) / (top - bottom);
        matrix[(2, 3)] = -z_near / (z_far - z_near);

        Self { matrix }
    }

    #[inline]
    pub fn top(&self) -> f32 {
        -((1.0 - self.matrix[(1, 3)]) / self.matrix[(1, 1)])
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        -((-1.0 - self.matrix[(1, 3)]) / self.matrix[(1, 1)])
    }

    #[inline]
    pub fn left(&self) -> f32 {
        (-1.0 - self.matrix[(0, 3)]) / self.matrix[(0, 0)]
    }

    #[inline]
    pub fn right(&self) -> f32 {
        (1.0 - self.matrix[(0, 3)]) / self.matrix[(0, 0)]
    }

    #[inline]
    pub fn near(&self) -> f32 {
        (self.matrix[(2, 3)] / self.matrix[(2, 2)])
    }

    #[inline]
    pub fn far(&self) -> f32 {
        ((-1.0 + self.matrix[(2, 3)]) / self.matrix[(2, 2)])
    }

    #[inline]
    pub fn set_top(&mut self, top: f32) {
        self.set_bottom_and_top(self.bottom(), top)
    }

    #[inline]
    pub fn set_bottom(&mut self, bottom: f32) {
        self.set_bottom_and_top(bottom, self.top())
    }

    #[inline]
    pub fn set_bottom_and_top(&mut self, bottom: f32, top: f32) {
        self.matrix[(1, 1)] = -2.0 / (top - bottom);
        self.matrix[(1, 3)] = -(top + bottom) / (top - bottom);
    }

    #[inline]
    pub fn set_left(&mut self, left: f32) {
        self.set_left_and_right(left, self.right())
    }

    #[inline]
    pub fn set_right(&mut self, right: f32) {
        self.set_left_and_right(self.left(), right)
    }

    #[inline]
    pub fn set_left_and_right(&mut self, left: f32, right: f32) {
        self.matrix[(0, 0)] = 2.0 / (right - left);
        self.matrix[(0, 3)] = -(right + left) / (right - left);
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        self.set_near_and_far(near, self.far())
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        self.set_near_and_far(self.near(), far)
    }

    #[inline]
    pub fn set_near_and_far(&mut self, z_near: f32, z_far: f32) {
        self.matrix[(2, 2)] = 1.0 / (z_far - z_near);
        self.matrix[(2, 3)] = -z_near / (z_far - z_near);
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
        if cfg!(debug_assertions) {
            assert!(
                !approx::relative_eq!(z_far - z_near, 0.0),
                "The near-plane and far-plane must not be superimposed."
            );
            assert!(
                !approx::relative_eq!(aspect, 0.0),
                "The apsect ratio must not be zero."
            );
        }

        let mut matrix = Matrix4::<f32>::zeros();
        let tan_half_fovy = (fov / 2.0).tan();

        matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        matrix[(1, 1)] = -1.0 / tan_half_fovy;
        matrix[(2, 2)] = z_far / (z_near - z_far);
        matrix[(2, 3)] = -(z_far * z_near) / (z_far - z_near);
        matrix[(3, 2)] = -1.0;

        Self { matrix }
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        (self.matrix[(1, 1)] / self.matrix[(0, 0)]).abs()
    }

    #[inline]
    pub fn fovy(&self) -> f32 {
        -(1.0 / self.matrix[(1, 1)]).atan() * 2.0
    }

    #[inline]
    pub fn near(&self) -> f32 {
        (self.matrix[(2, 3)] / self.matrix[(2, 2)])
    }

    #[inline]
    pub fn far(&self) -> f32 {
        self.matrix[(2, 3)] / (self.matrix[(2, 2)] + 1.0)
    }

    #[inline]
    pub fn set_aspect(&mut self, aspect: f32) {
        let tan_half_fovy = (self.fovy() / 2.0).tan();
        self.matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
    }

    #[inline]
    pub fn set_fov(&mut self, fov: f32) {
        let tan_half_fovy = (fov / 2.0).tan();
        self.matrix[(0, 0)] = 1.0 / (self.aspect() * tan_half_fovy);
        self.matrix[(1, 1)] = -1.0 / tan_half_fovy;
    }

    #[inline]
    pub fn set_fov_and_aspect(&mut self, fov: f32, aspect: f32) {
        let tan_half_fovy = (fov / 2.0).tan();
        self.matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        self.matrix[(1, 1)] = -1.0 / tan_half_fovy;
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        self.set_near_and_far(near, self.far())
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        self.set_near_and_far(self.near(), far)
    }

    #[inline]
    pub fn set_near_and_far(&mut self, z_near: f32, z_far: f32) {
        self.matrix[(2, 2)] = z_far / (z_near - z_far);
        self.matrix[(2, 3)] = -(z_near * z_far) / (z_far - z_near);
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
    Orthographic(Orthographic),
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective(Perspective),
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

    pub fn as_orthographic(&self) -> Option<&Orthographic> {
        match *self {
            Projection::Orthographic(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_orthographic_mut(&mut self) -> Option<&mut Orthographic> {
        match *self {
            Projection::Orthographic(ref mut s) => Some(s),
            _ => None,
        }
    }

    pub fn as_perspective(&self) -> Option<&Perspective> {
        match *self {
            Projection::Perspective(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_perspective_mut(&mut self) -> Option<&mut Perspective> {
        match *self {
            Projection::Perspective(ref mut s) => Some(s),
            _ => None,
        }
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref s) => s.as_matrix(),
            Projection::Perspective(ref s) => s.as_matrix(),
        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref mut s) => s.as_matrix_mut(),
            Projection::Perspective(ref mut s) => s.as_matrix_mut(),
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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
            Projection::Orthographic(ref p) => p.as_matrix(),
            Projection::Perspective(ref p) => p.as_matrix(),
        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match self.inner {
            Projection::Orthographic(ref mut p) => p.as_matrix_mut(),
            Projection::Perspective(ref mut p) => p.as_matrix_mut(),
        }
    }

    pub fn projection(&self) -> &Projection {
        &self.inner
    }

    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.inner
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
pub enum CameraPrefab {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    },
    Perspective {
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    },
}

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
        storage.insert(
            entity,
            Camera {
                inner: match *self {
                    CameraPrefab::Orthographic {
                        left,
                        right,
                        bottom,
                        top,
                        znear,
                        zfar,
                    } => Projection::orthographic(left, right, bottom, top, znear, zfar),
                    CameraPrefab::Perspective {
                        aspect,
                        fovy,
                        znear,
                        zfar,
                    } => Projection::perspective(aspect, fovy, znear, zfar),
                },
            },
        )?;
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

#[cfg(test)]
mod tests {
    //! Tests for amethysts camera implementation.
    //!
    //! Assertions are in NDC
    //! Our world-space is +Y Up, +X Right and -Z Away
    //! Our view space is +Y Down, +X Right, +Z Away
    //! Current render target is +Y Down, +X Right, +Z Away

    use super::*;
    use amethyst_core::{
        math::{
            convert, Isometry3, Matrix4, Point3, Translation3, UnitQuaternion, Vector3, Vector4,
        },
        Transform,
    };
    use ron::{de::from_str, ser::to_string_pretty};

    use approx::{assert_abs_diff_eq, assert_relative_eq, assert_ulps_eq};
    use more_asserts::{assert_ge, assert_gt, assert_le, assert_lt};

    #[test]
    #[ignore]
    fn test_orthographic_serde() {
        let test_ortho = Projection::orthographic(0.0, 100.0, 10.0, 150.0, -5.0, 100.0);
        println!(
            "{}",
            to_string_pretty(&test_ortho, Default::default()).unwrap()
        );

        let de = from_str(&to_string_pretty(&test_ortho, Default::default()).unwrap()).unwrap();
        assert_eq!(test_ortho, de);
    }

    #[test]
    #[ignore]
    fn test_perspective_serde() {
        let test_persp = Projection::perspective(1.7, std::f32::consts::FRAC_PI_3, 0.1, 1000.0);
        println!(
            "{}",
            to_string_pretty(&test_persp, Default::default()).unwrap()
        );

        let de = from_str(&to_string_pretty(&test_persp, Default::default()).unwrap()).unwrap();

        assert_eq!(test_persp, de);
    }

    #[test]
    fn extract_perspective_values() {
        let proj = Perspective::new(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);

        assert_ulps_eq!(1280.0 / 720.0, proj.aspect());
        assert_ulps_eq!(std::f32::consts::FRAC_PI_3, proj.fovy());
        assert_ulps_eq!(0.1, proj.near());
        // TODO: we need to solve these precision errors
        assert_relative_eq!(100.0, proj.far(), max_relative = 1.0);

        //let proj = Projection::perspective(width/height, std::f32::consts::FRAC_PI_3, 0.1, 2000.0);
        let proj_standard = Camera::standard_3d(1920.0, 1280.0);
        assert_ulps_eq!(
            std::f32::consts::FRAC_PI_3,
            proj_standard.projection().as_perspective().unwrap().fovy()
        );
        assert_ulps_eq!(
            1.5,
            proj_standard
                .projection()
                .as_perspective()
                .unwrap()
                .aspect()
        );
        assert_ulps_eq!(
            0.1,
            proj_standard.projection().as_perspective().unwrap().near()
        );
        assert_relative_eq!(
            2000.0,
            proj_standard.projection().as_perspective().unwrap().far(),
            max_relative = 3.0
        );
    }

    #[test]
    fn extract_orthographic_values() {
        let proj = Orthographic::new(0.0, 100.0, 10.0, 150.0, -5.0, 100.0);

        // TODO: we need to solve these precision errors
        assert_ulps_eq!(150.0, proj.top());
        assert_ulps_eq!(10.0, proj.bottom());

        assert_ulps_eq!(0.0, proj.left());
        assert_ulps_eq!(100.0, proj.right());
        assert_ulps_eq!(-5.0, proj.near());
        assert_relative_eq!(100.0, proj.far(), max_relative = 0.1);

        let camera_standard = Camera::standard_2d(1920.0, 1280.0);

        // TODO: we need to solve these precision errors
        assert_relative_eq!(
            -640.0,
            camera_standard
                .projection()
                .as_orthographic()
                .unwrap()
                .bottom(),
            max_relative = 1.0
        );
        assert_relative_eq!(
            640.0,
            camera_standard
                .projection()
                .as_orthographic()
                .unwrap()
                .top(),
            max_relative = 1.0
        );
        assert_relative_eq!(
            -960.0,
            camera_standard
                .projection()
                .as_orthographic()
                .unwrap()
                .left(),
            max_relative = 1.0
        );
        assert_relative_eq!(
            960.0,
            camera_standard
                .projection()
                .as_orthographic()
                .unwrap()
                .right(),
            max_relative = 1.0
        );
        assert_relative_eq!(
            0.1,
            camera_standard
                .projection()
                .as_orthographic()
                .unwrap()
                .near(),
            max_relative = 1.0
        );
        //assert_ulps_eq!(2000.0, camera_standard.projection().as_orthographic().unwrap().far());
    }

    // Our world-space is +Y Up, +X Right and -Z Away
    // Current render target is +Y Down, +X Right and +Z Away
    fn setup() -> (Transform, [Point3<f32>; 3], [Point3<f32>; 3]) {
        // Setup common inputs for most of the tests.
        //
        // Sets up a test camera is positioned at (0,0,3) in world space.
        // A camera without rotation is pointing in the (0,0,-1) direction.
        //
        // Sets up basic points.
        let camera_transform: Transform = Transform::new(
            Translation3::new(0.0, 0.0, 3.0),
            // Apply _no_ rotation
            UnitQuaternion::identity(),
            [1.0, 1.0, 1.0].into(),
        );

        let simple_points: [Point3<f32>; 3] = [
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
        ];

        let simple_points_clipped: [Point3<f32>; 3] = [
            Point3::new(-20.0, 0.0, 0.0),
            Point3::new(0.0, -20.0, 0.0),
            Point3::new(0.0, 0.0, 4.0),
        ];
        (camera_transform, simple_points, simple_points_clipped)
    }

    fn gatherer_calc_view_matrix(transform: Transform) -> Matrix4<f32> {
        convert(transform.view_matrix())
    }

    #[test]
    fn camera_matrix() {
        let (camera_transform, simple_points, _) = setup();
        let view_matrix = Isometry3::look_at_rh(
            &Point3::new(0.0, 0.0, 3.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::y_axis(),
        );

        // Check view matrix.
        // The view matrix is used to transfrom a point from world space to eye space.
        // Changes the base of a vector from world origin to your eye.
        let our_view: Matrix4<f32> = gatherer_calc_view_matrix(camera_transform);
        assert_ulps_eq!(our_view, view_matrix.to_homogeneous(),);

        let x_axis = our_view * simple_points[0].to_homogeneous();
        let y_axis = our_view * simple_points[1].to_homogeneous();
        let z_axis = our_view * simple_points[2].to_homogeneous();
        assert_gt!(x_axis[0], 0.0);
        assert_gt!(y_axis[1], 0.0);
        assert_le!(z_axis[2], 0.0);
    }

    #[test]
    fn standard_2d() {
        let width = 1280.0;
        let height = 720.0;
        let top = height / 2.0;
        let bottom = -height / 2.0;
        let left = -width / 2.0;
        let right = width / 2.0;

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
        let proj =
            Projection::perspective(width / height, std::f32::consts::FRAC_PI_3, 0.1, 2000.0);
        let our_proj = Camera::standard_3d(width, height).inner;

        assert_ulps_eq!(our_proj.as_matrix(), proj.as_matrix());
    }

    #[test]
    fn perspective_orientation() {
        // -w_c <= x_c <= w_c
        // -w_c <= y_c <= w_c
        // 0 <= z_c <= w_c
        //
        // https://www.khronos.org/registry/vulkan/specs/1.0/html/vkspec.html#vertexpostproc-clipping-shader-outputs
        let (camera_transform, simple_points, simple_points_clipped) = setup();

        let proj = Projection::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let x_axis = mvp * simple_points[0].to_homogeneous();
        let y_axis = mvp * simple_points[1].to_homogeneous();
        let z_axis = mvp * simple_points[2].to_homogeneous();

        assert_gt!(x_axis[0], 0.0);
        assert_gt!(x_axis[0] / x_axis[3], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);
        assert_lt!(y_axis[1] / y_axis[3], 0.0);

        // Z should be in [0; w] resp. [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_ge!(z_axis[2] / z_axis[3], 0.0);
        assert_le!(z_axis[2], z_axis[3]);
        assert_le!(z_axis[2] / z_axis[3], 1.0);

        let x_axis_clipped = mvp * simple_points_clipped[0].to_homogeneous();
        let y_axis_clipped = mvp * simple_points_clipped[1].to_homogeneous();
        let z_axis_clipped = mvp * simple_points_clipped[2].to_homogeneous();

        // Outside of frustum should be clipped (Test in Clipspace)
        assert_le!(x_axis_clipped[0], -1.0);
        assert_ge!(y_axis_clipped[1], 1.0);

        // Behind Camera should be clipped. (Test in Clipspace)
        assert_lt!(z_axis_clipped[2], 0.0);
    }

    // Todo: Add perspective_orientation_reversed_z when we support reversed z depth buffer.

    #[test]
    fn orthographic_orientation() {
        let (camera_transform, simple_points, _) = setup();

        let proj = Projection::orthographic(
            -1280.0 / 2.0,
            1280.0 / 2.0,
            -720.0 / 2.0,
            720.0 / 2.0,
            0.1,
            100.0,
        );
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let x_axis = mvp * simple_points[0].to_homogeneous();
        let y_axis = mvp * simple_points[1].to_homogeneous();
        let z_axis = mvp * simple_points[2].to_homogeneous();

        assert_gt!(x_axis[0], 0.0);
        assert_gt!(x_axis[0] / x_axis[3], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);
        assert_lt!(y_axis[1] / y_axis[3], 0.0);

        // Z should be in [0; w] resp. [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_ge!(z_axis[2] / z_axis[3], 0.0);
        assert_le!(z_axis[2], z_axis[3]);
        assert_le!(z_axis[2] / z_axis[3], 1.0);
    }

    #[test]
    fn perspective_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Projection::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;
        // Nearest point = distance to (0,0) - zNear
        let near = Point3::new(0.0, 0.0, 2.9);
        let projected_point = mvp * near.to_homogeneous();
        assert_abs_diff_eq!(projected_point[2] / projected_point[3], 0.0);

        // Furthest point = distance to (0,0) - zFar
        let far = Point3::new(0.0, 0.0, -97.0);
        let projected_point = mvp * far.to_homogeneous();
        assert_abs_diff_eq!(projected_point[2] / projected_point[3], 1.0);
    }

    #[test]
    fn orthographic_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Projection::orthographic(
            -1280.0 / 2.0,
            1280.0 / 2.0,
            -720.0 / 2.0,
            720.0 / 2.0,
            0.1,
            100.0,
        );
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;
        // Nearest point = distance to (0,0) - zNear
        let near = Point3::new(0.0, 0.0, 2.9);
        let projected_point = mvp * near.to_homogeneous();
        assert_abs_diff_eq!(projected_point[2] / projected_point[3], 0.0);

        // Furthest point = distance to (0,0) - zFar
        let far = Point3::new(0.0, 0.0, -97.0);
        let projected_point = mvp * far.to_homogeneous();
        assert_abs_diff_eq!(projected_point[2] / projected_point[3], 1.0);
    }

    #[test]
    #[ignore]
    fn perspective_project_cube_centered() {
        let (camera_transform, _, _) = setup();

        // Cube in worldspace
        let cube = [
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(1.0, 1.0, -1.0),
            Point3::new(-1.0, 1.0, -1.0),
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, -1.0, -1.0),
        ];

        let proj = Projection::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.as_matrix() * view;

        let _result: Vec<Vector4<f32>> = cube
            .into_iter()
            .map(|vertex| mvp * vertex.to_homogeneous())
            .collect();
        // TODO: Calc correct result
        // assert_ulps_eq!(result, Point());
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn perspective_project_cube_off_centered_rotated() {
        let (camera_transform, _, _) = setup();
        // Cube in worldspace
        let cube = [
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(1.0, 1.0, -1.0),
            Point3::new(-1.0, 1.0, -1.0),
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, -1.0, -1.0),
        ];
        let proj = Projection::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1, 100.0);
        let view = gatherer_calc_view_matrix(camera_transform);

        // Rotated x and y axis by 45°
        let rotation = UnitQuaternion::from_euler_angles(
            std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        );
        let model =
            Isometry3::from_parts(Translation3::new(-1.0, 0.0, 0.0), rotation).to_homogeneous();

        let mvp = proj.as_matrix() * view * model;

        // Todo: Maybe check more cells of the model matrix
        assert_ulps_eq!(model.column(0)[0], 0.70710678118);
        assert_ulps_eq!(model.column(3)[0], -1.0);

        let _result: Vec<Vector4<f32>> = cube
            .iter()
            .map(|vertex| mvp * vertex.to_homogeneous())
            .collect();

        // TODO: Calc correct result
        // assert_ulps_eq!(result, Point());
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn orthographic_project_cube_off_centered_rotated() {
        unimplemented!()
    }
}
