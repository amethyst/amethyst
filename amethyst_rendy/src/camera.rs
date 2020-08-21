//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
    geometry::Ray,
    math::{Matrix4, Point2, Point3, Vector2},
    transform::components::Transform,
};
use amethyst_error::Error;

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
///  -z
/// /
/// |¯¯¯+x
/// |
/// +y
///
/// The camera also stores the inverse transformation in order to avoid recomputing it.
///
/// If you change `matrix` you must also change `inverse` so that they stay in sync.
/// You should probably use from_matrix instead.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Camera {
    /// The projection matrix
    pub matrix: Matrix4<f32>,
    /// Its inverse
    pub inverse: Matrix4<f32>,
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
        Self::orthographic(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
            0.125,
            2000.0,
        )
    }

    /// An appropriate orthographic projection for the coordinate space used by Amethyst.
    /// Because we use vulkan coordinates internally and within the rendering engine, normal nalgebra
    /// projection objects (`Orthographic3` are incorrect for our use case.
    ///
    /// The projection matrix is right-handed and depth goes from 1 to 0.
    ///
    /// # Arguments
    ///
    /// * `left` - The x-coordinate of the cuboid leftmost face parallel to the yz-plane.
    /// * `right` - The x-coordinate of the cuboid rightmost face parallel to the yz-plane.
    /// * `top` - The upper y-coordinate of the cuboid leftmost face parallel to the xz-plane.
    /// * `bottom` - The lower y-coordinate of the cuboid leftmost face parallel to the xz-plane.
    /// * `z_near` - The distance between the viewer (the origin) and the closest face of the cuboid parallel to the xy-plane. If used for a 3D rendering application, this is the closest clipping plane.
    /// * `z_far` - The distance between the viewer (the origin) and the furthest face of the cuboid parallel to the xy-plane. If used for a 3D rendering application, this is the furthest clipping plane.
    ///
    /// * panics if `left` equals `right`, `bottom` equals `top` or `z_near` equals `z_far`
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        if cfg!(debug_assertions) {
            assert!(
                !approx::relative_eq!(z_far - z_near, 0.0),
                "The near-plane and far-plane must not be superimposed."
            );
            assert!(
                !approx::relative_eq!(left - right, 0.0),
                "The left-plane and right-plane must not be superimposed."
            );
            assert!(
                !approx::relative_eq!(top - bottom, 0.0),
                "The top-plane and bottom-plane must not be superimposed."
            );
        }

        let mut matrix = Matrix4::<f32>::identity();

        matrix[(0, 0)] = 2.0 / (right - left);
        matrix[(1, 1)] = -2.0 / (top - bottom);
        matrix[(2, 2)] = 1.0 / (z_far - z_near);
        matrix[(0, 3)] = -(right + left) / (right - left);
        matrix[(1, 3)] = -(top + bottom) / (top - bottom);
        matrix[(2, 3)] = z_far / (z_far - z_near);

        // TODO: build the inverse instead of using generic matrix inversion
        Self::from_matrix(matrix)
    }

    /// Create a standard camera for 3D.
    ///
    /// Will use a perspective projection with aspect from the given screen dimensions and a field
    /// of view of π/3 radians (60 degrees).
    /// View transformation will be multiplicative identity.
    pub fn standard_3d(width: f32, height: f32) -> Self {
        Self::perspective(width / height, std::f32::consts::FRAC_PI_3, 0.125)
    }

    /// An appropriate perspective projection for the coordinate space used by Amethyst.
    /// Because we use vulkan coordinates internally and within the rendering engine, normal nalgebra
    /// projection objects (`Perspective3`) are incorrect for our use case.
    ///
    /// The projection matrix is right-handed and depth goes from 1 to 0.
    ///
    /// # Arguments
    ///
    /// * aspect - Aspect Ratio represented as a `f32` ratio.
    /// * fov - Field of View represented in radians
    /// * z_near - Near clip plane distance
    ///
    /// * panics when matrix is not invertible
    pub fn perspective(aspect: f32, fov: f32, z_near: f32) -> Self {
        if cfg!(debug_assertions) {
            assert!(
                !approx::relative_eq!(aspect, 0.0),
                "The aspect ratio must not be zero."
            );
        }

        let mut matrix = Matrix4::<f32>::zeros();
        let tan_half_fovy = (fov / 2.0).tan();

        matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        matrix[(1, 1)] = -1.0 / tan_half_fovy;
        matrix[(2, 3)] = z_near;
        matrix[(3, 2)] = -1.0;

        // TODO: is there an optimized way to compute the inverse of this matrix?
        Self::from_matrix(matrix)
    }

    /// Makes a camera with the matrix provided.
    ///
    /// * panics if the matrix is not invertible
    pub fn from_matrix(matrix: Matrix4<f32>) -> Self {
        Self{
            matrix,
            inverse: matrix
                .try_inverse()
                .expect("Camera projection matrix is not invertible. This is normally due to having inverse values being superimposed (near=far, right=left)")
        }
    }

    /// Returns a `Ray` going out form the camera through provided screen position. The ray origin lies on camera near plane.
    ///
    /// The screen coordinate (0, 0) is the top-left corner of the top-left pixel.
    /// `screen_diagonal` is the bottom-right corner of the bottom-right pixel.
    pub fn screen_ray(
        &self,
        screen_position: Point2<f32>,
        screen_diagonal: Vector2<f32>,
        camera_transform: &Transform,
    ) -> Ray<f32> {
        let screen_x = 2.0 * screen_position.x / screen_diagonal.x - 1.0;
        let screen_y = 2.0 * screen_position.y / screen_diagonal.y - 1.0;

        let matrix = *camera_transform.global_matrix() * self.inverse;

        let near = Point3::new(screen_x, screen_y, 1.0);
        // The constraint on far is: 0.0 < far < 1.0. We arbitrarily chose 0.5 - maybe there is a better value?
        let far = Point3::new(screen_x, screen_y, 0.5);

        let near_t = matrix.transform_point(&near);
        let far_t = matrix.transform_point(&far);

        Ray {
            origin: near_t,
            direction: (far_t.coords - near_t.coords).normalize(),
        }
    }

    /// Transforms the provided (X, Y, Z) screen coordinate into world coordinates.
    /// This method fires a ray from the camera in its view direction, and returns the Point at `screen_position.z`
    /// world space distance from the camera origin.
    pub fn screen_to_world_point(
        &self,
        screen_position: Point3<f32>,
        screen_diagonal: Vector2<f32>,
        camera_transform: &Transform,
    ) -> Point3<f32> {
        self.screen_ray(screen_position.xy(), screen_diagonal, camera_transform)
            .at_distance(screen_position.z)
    }

    /// Translate from world coordinates to screen coordinates
    ///
    /// The screen coordinate (0, 0) is the top-left corner of the top-left pixel.
    /// `screen_diagonal` is the bottom-right corner of the bottom-right pixel.
    pub fn world_to_screen(
        &self,
        world_position: Point3<f32>,
        screen_diagonal: Vector2<f32>,
        camera_transform: &Transform,
    ) -> Point2<f32> {
        let transformation_matrix = camera_transform.global_matrix().try_inverse().unwrap();
        let screen_pos = (self.matrix * transformation_matrix).transform_point(&world_position);

        Point2::new(
            (screen_pos.x + 1.0) * screen_diagonal.x / 2.0,
            (screen_pos.y + 1.0) * screen_diagonal.y / 2.0,
        )
    }
}

impl PartialEq for Camera {
    fn eq(&self, other: &Camera) -> bool {
        self.matrix == other.matrix
    }
}

impl Component for Camera {
    type Storage = HashMapStorage<Self>;
}

/// Active camera resource, used by the renderer to choose which camera to get the view matrix from.
/// If no active camera is found, the first camera will be used as a fallback.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActiveCamera {
    /// Camera entity
    pub entity: Option<Entity>,
}

/// Projection prefab
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum CameraPrefab {
    /// Orthographic prefab
    Orthographic {
        /// The x-coordinate of the cuboid leftmost face parallel to the yz-plane.
        left: f32,
        /// The x-coordinate of the cuboid rightmost face parallel to the yz-plane.
        right: f32,
        /// The lower y-coordinate of the cuboid leftmost face parallel to the xz-plane.
        bottom: f32,
        /// The upper y-coordinate of the cuboid leftmost face parallel to the xz-plane.
        top: f32,
        /// The distance between the viewer (the origin) and the closest face of the cuboid parallel to the xy-plane. If used for a 3D rendering application, this is the closest clipping plane.
        znear: f32,
        /// The distance between the viewer (the origin) and the furthest face of the cuboid parallel to the xy-plane. If used for a 3D rendering application, this is the furthest clipping plane.
        zfar: f32,
    },
    /// Perspective prefab
    Perspective {
        /// Aspect Ratio represented as a `f32` ratio.
        aspect: f32,
        /// Field of View represented in degrees
        fovy: f32,
        /// Near clip plane distance
        znear: f32,
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
            match *self {
                CameraPrefab::Orthographic {
                    left,
                    right,
                    bottom,
                    top,
                    znear,
                    zfar,
                } => Camera::orthographic(left, right, bottom, top, znear, zfar),
                CameraPrefab::Perspective {
                    aspect,
                    fovy,
                    znear,
                } => Camera::perspective(aspect, fovy, znear),
            },
        )?;
        Ok(())
    }
}

/// Active camera prefab
#[derive(Debug, serde::Deserialize, Clone)]
pub struct ActiveCameraPrefab(Option<usize>);

impl<'a> PrefabData<'a> for ActiveCameraPrefab {
    type SystemData = (Write<'a, ActiveCamera>,);
    type Result = ();

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        if let Some(ref ent) = self.0 {
            system_data.0.entity = Some(entities[*ent]);
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
        math::{convert, Isometry3, Matrix4, Point3, Translation3, UnitQuaternion, Vector3},
        Transform,
    };
    use ron::{de::from_str, ser::to_string_pretty};

    use approx::{assert_abs_diff_eq, assert_ulps_eq};
    use more_asserts::{assert_ge, assert_gt, assert_le, assert_lt};

    #[test]
    fn screen_to_world_3d() {
        let diagonal = Vector2::new(1024.0, 768.0);

        let camera = Camera::standard_3d(diagonal.x, diagonal.y);
        let mut transform = Transform::default();

        let center_screen = Point3::new(diagonal.x / 2.0, diagonal.y / 2.0, 0.0);
        let top_left = Point3::new(0.0, 0.0, 0.0);
        let bottom_right = Point3::new(diagonal.x, diagonal.y, 0.0);

        assert_ulps_eq!(
            camera.screen_to_world_point(center_screen, diagonal, &transform),
            Point3::new(0.0, 0.0, -0.125)
        );

        // y is tan(fov/2) * near and x is that times aspect ratio

        assert_ulps_eq!(
            camera.screen_to_world_point(top_left, diagonal, &transform),
            Point3::new(-0.09622504486493762, 0.07216878364870322, -0.125)
        );

        assert_ulps_eq!(
            camera.screen_to_world_point(bottom_right, diagonal, &transform),
            Point3::new(0.09622504486493762, -0.07216878364870322, -0.125)
        );

        transform.set_translation_x(100.0);
        transform.set_translation_y(100.0);
        transform.copy_local_to_global();
        assert_ulps_eq!(
            camera.screen_to_world_point(center_screen, diagonal, &transform),
            Point3::new(100.0, 100.0, -0.125)
        );
    }

    #[test]
    fn screen_to_world_2d() {
        let diagonal = Vector2::new(1024.0, 768.0);

        let camera = Camera::standard_2d(diagonal.x, diagonal.y);
        let mut transform = Transform::default();

        let center_screen = Point3::new(diagonal.x / 2.0, diagonal.y / 2.0, 0.0);
        let top_left = Point3::new(0.0, 0.0, 0.0);
        let bottom_right = Point3::new(diagonal.x, diagonal.y, 0.0);

        assert_ulps_eq!(
            camera.screen_to_world_point(center_screen, diagonal, &transform),
            Point3::new(0.0, 0.0, -0.125)
        );

        assert_ulps_eq!(
            camera.screen_to_world_point(top_left, diagonal, &transform),
            Point3::new(-diagonal.x / 2.0, diagonal.y / 2.0, -0.125)
        );

        assert_ulps_eq!(
            camera.screen_to_world_point(bottom_right, diagonal, &transform),
            Point3::new(diagonal.x / 2.0, -diagonal.y / 2.0, -0.125)
        );

        transform.set_translation_x(100.0);
        transform.set_translation_y(100.0);
        transform.copy_local_to_global();
        assert_ulps_eq!(
            camera.screen_to_world_point(center_screen, diagonal, &transform),
            Point3::new(100.0, 100.0, -0.125)
        );
    }

    #[test]
    fn world_to_screen() {
        let diagonal = Vector2::new(1024.0, 768.0);

        let ortho = Camera::standard_2d(diagonal.x, diagonal.y);
        let transform = Transform::default();

        let center_screen = Point2::new(diagonal.x / 2.0, diagonal.y / 2.0);
        let top_left = Point2::new(0.0, 0.0);
        let bottom_right = Point2::new(diagonal.x, diagonal.y);

        let top_left_world = Point3::new(-diagonal.x / 2.0, diagonal.y / 2.0, -0.1);
        let bottom_right_world = Point3::new(diagonal.x / 2.0, -diagonal.y / 2.0, -0.1);

        assert_ulps_eq!(
            ortho.world_to_screen(top_left_world, diagonal, &transform),
            top_left
        );

        assert_ulps_eq!(
            ortho.world_to_screen(bottom_right_world, diagonal, &transform),
            bottom_right
        );

        assert_ulps_eq!(
            ortho.world_to_screen(Point3::new(0.0, 0.0, 0.0), diagonal, &transform),
            center_screen
        );
    }

    #[test]
    fn test_orthographic_serde() {
        let test_ortho = Camera::orthographic(0.0, 100.0, 10.0, 150.0, -5.0, 100.0);
        println!(
            "{}",
            to_string_pretty(&test_ortho, Default::default()).unwrap()
        );

        let de = from_str(&to_string_pretty(&test_ortho, Default::default()).unwrap()).unwrap();
        assert_eq!(test_ortho, de);
    }

    #[test]
    fn test_perspective_serde() {
        let test_persp = Camera::perspective(1.7, std::f32::consts::FRAC_PI_3, 0.1);
        println!(
            "{}",
            to_string_pretty(&test_persp, Default::default()).unwrap()
        );

        let de = from_str(&to_string_pretty(&test_persp, Default::default()).unwrap()).unwrap();

        assert_eq!(test_persp, de);
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
        let proj = Camera::orthographic(left, right, bottom, top, 0.125, 2000.0);
        let our_proj = Camera::standard_2d(width, height);

        assert_ulps_eq!(our_proj.matrix, proj.matrix);
    }

    #[test]
    fn standard_3d() {
        let width = 1280.0;
        let height = 720.0;

        let proj = Camera::perspective(width / height, std::f32::consts::FRAC_PI_3, 0.125);
        let our_proj = Camera::standard_3d(width, height);

        assert_ulps_eq!(our_proj.matrix, proj.matrix);
    }

    #[test]
    fn perspective_orientation() {
        // -w_c <= x_c <= w_c
        // -w_c <= y_c <= w_c
        // 0 <= z_c <= w_c
        //
        // https://www.khronos.org/registry/vulkan/specs/1.0/html/vkspec.html#vertexpostproc-clipping-shader-outputs
        let (camera_transform, simple_points, simple_points_clipped) = setup();

        let proj = Camera::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 0.1);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.matrix * view;

        let x_axis = mvp.transform_point(&simple_points[0]);
        let y_axis = mvp.transform_point(&simple_points[1]);
        let z_axis = mvp.transform_point(&simple_points[2]);

        assert_gt!(x_axis[0], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);

        // Z should be in [0; w] resp. [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_le!(z_axis[2], 1.0);

        let x_axis_clipped = mvp.transform_point(&simple_points_clipped[0]);
        let y_axis_clipped = mvp.transform_point(&simple_points_clipped[1]);
        let z_axis_clipped = mvp.transform_point(&simple_points_clipped[2]);

        // Outside of frustum should be clipped (Test in Clipspace)
        assert_le!(x_axis_clipped[0], -1.0);
        assert_ge!(y_axis_clipped[1], 1.0);

        // Behind Camera should be clipped. (Test in Clipspace)
        assert_lt!(z_axis_clipped[2], 0.0);
    }

    #[test]
    fn orthographic_orientation() {
        let (camera_transform, simple_points, _) = setup();

        let proj = Camera::orthographic(
            -1280.0 / 2.0,
            1280.0 / 2.0,
            -720.0 / 2.0,
            720.0 / 2.0,
            0.1,
            100.0,
        );
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.matrix * view;

        let x_axis = mvp.transform_point(&simple_points[0]);
        let y_axis = mvp.transform_point(&simple_points[1]);
        let z_axis = mvp.transform_point(&simple_points[2]);

        assert_gt!(x_axis[0], 0.0);

        // Y should be negative
        assert_lt!(y_axis[1], 0.0);

        // Z should be in [0; w] resp. [0; 1]
        assert_ge!(z_axis[2], 0.0);
        assert_le!(z_axis[2], 1.0);
    }

    #[test]
    fn perspective_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Camera::perspective(1280.0 / 720.0, std::f32::consts::FRAC_PI_3, 1.);
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.matrix * view;
        // Nearest point = distance to (0,0) - zNear
        let near = Point3::new(0.0, 0.0, 2.);
        let projected_point = mvp.transform_point(&near);
        assert_abs_diff_eq!(projected_point[2], 1.0);

        // Furthest point = distance to (0,0) - zFar
        let far = Point3::new(0.0, 0.0, -1_000_000_000.0);
        let projected_point = mvp.transform_point(&far);
        assert_abs_diff_eq!(projected_point[2], 0.0);
    }

    #[test]
    fn orthographic_depth_usage() {
        let (camera_transform, _, _) = setup();

        let proj = Camera::orthographic(
            -1280.0 / 2.0,
            1280.0 / 2.0,
            -720.0 / 2.0,
            720.0 / 2.0,
            0.1,
            100.0,
        );
        let view = gatherer_calc_view_matrix(camera_transform);

        let mvp = proj.matrix * view;
        // Nearest point = distance to (0,0) - zNear
        let near = Point3::new(0.0, 0.0, 2.9);
        let projected_point = mvp.transform_point(&near);
        assert_abs_diff_eq!(projected_point[2], 1.0);

        // Furthest point = distance to (0,0) - zFar
        let far = Point3::new(0.0, 0.0, -97.0);
        let projected_point = mvp.transform_point(&far);
        assert_abs_diff_eq!(projected_point[2], 0.0);
    }

    #[test]
    fn screen_ray_3d() {
        let width = 1280.0;
        let height = 720.0;

        let aspect = width / height;
        let fov = std::f32::consts::FRAC_PI_3;
        let znear = 0.125;
        let camera = Camera::perspective(aspect, fov, znear);

        let mut camera_transform: Transform = Transform::new(
            Translation3::new(0.0, 0.0, 3.0),
            UnitQuaternion::identity(),
            [1.0, 1.0, 1.0].into(),
        );
        camera_transform.copy_local_to_global();

        let cursor_pos = Point2::new(width, height);
        let screen_diag = Vector2::new(width, height);
        let ray = camera.screen_ray(cursor_pos, screen_diag, &camera_transform);

        let expected_ray = Ray {
            /// In the znear plane.
            origin: Point3::new(0.12830007, -0.0721688, 2.875),
            // Corresponds to 45 degree rotation to the right and 30 degree rotation up. This ray
            // should pass through the top right corner of the screen.
            direction: Vector3::new(0.6643638, -0.37370467, -0.6472754),
        };
        assert_ulps_eq!(ray.origin, expected_ray.origin);
        assert_ulps_eq!(ray.direction, expected_ray.direction);
    }
}
