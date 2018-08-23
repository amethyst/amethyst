use cgmath::{Angle, Deg, Matrix2, SquareMatrix, Vector2};
use specs::{Component, DenseVecStorage, FlaggedStorage};

/// Component describing the position of an entity on an isometric plane.
/// This Transform will place the entity according to isometric coordinates
/// on the local plane to abstract away isometric math.
/// The transform converts the isometric coordinates into local
/// coordinates and insert them into the associated `Transform`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct IsometricTransform {
    /// Position of the transform on the isometric plane.
    pub translation: Vector2<f32>,
    /// Weight influencing the likelihood of this entity to be rendered
    /// on top of other entities. Entities are ordered by `-local_y + priority`.
    /// The higher the priority, the higher the likelihood of this entity
    /// to be rendered on top.
    pub order_priority: f32,

    scale: Vector2<f32>,
    angle: Deg<f32>,
    iso_to_local: Matrix2<f32>,
    local_to_iso: Matrix2<f32>,
}

fn matrices_from_angle(angle: Deg<f32>, scale: Vector2<f32>) -> (Matrix2<f32>, Matrix2<f32>) {
    // This is a change of basis matrix.
    // It goes from the usual basis to the isometric basis.
    let local_to_iso = Matrix2 {
        x: Vector2 {
            x: scale.x * angle.cos(),
            y: scale.x * angle.cos(),
        },
        y: Vector2 {
            x: scale.y * angle.cos(),
            y: -scale.y * angle.cos(),
        },
    };

    // The error handling is just a panic because having the parameters
    // of the isometry be dynamic seems unrealistic.
    // Can quite easily be changed if need be.
    let iso_to_local = local_to_iso.invert().unwrap_or_else(|| {
        panic!(
            "invalid angle {:?} or scale ({:?};{:?}) for IsometricTransform",
            angle, scale.x, scale.y
        )
    });

    (local_to_iso, iso_to_local)
}

impl Default for IsometricTransform {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        let (local_to_iso, iso_to_local) =
            matrices_from_angle(Deg(30.0), Vector2 { x: 1.0, y: 1.0 });
        Self {
            translation: Vector2 { x: 0.0, y: 0.0 },
            order_priority: 0.0,
            scale: Vector2 { x: 1.0, y: 1.0 },
            angle: Deg(30.0),
            local_to_iso,
            iso_to_local,
        }
    }
}

impl Component for IsometricTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl IsometricTransform {
    /// Builds an IsometricTransform from the local dimensions of a unit.
    /// In other words, if you consider an isometric square unit, this function
    /// will build the isometric grid so that this unit fits in a rectangle of
    /// size (w;h) in the local coordinates system.
    pub fn from_unit_dimensions(w: f32, h: f32) -> Self {
        let angle = Deg((h / w).atan());
        let scale = Vector2 {
            x: 1.0 / w,
            y: -1.0 / h,
        };
        Self::from_angle(angle, scale)
    }

    /// Builds an IsometricTransform from the angle between the usual x axis
    /// and the isometric axes. In other words, if you consider an isometric unit,
    /// the angle parameter is half the angle of the lateral corners.
    pub fn from_angle(angle: Deg<f32>, scale: Vector2<f32>) -> Self {
        let (local_to_iso, iso_to_local) = matrices_from_angle(angle, scale);
        Self {
            translation: Vector2 { x: 0.0, y: 0.0 },
            order_priority: 0.0,
            scale,
            angle,
            local_to_iso,
            iso_to_local,
        }
    }

    /// Returns the isometric translation vector in this transform's
    /// plane of a local translation vector. In other words, this
    /// converts local coordinates into isometric coordinates.
    pub fn local_to_iso(&self, local: Vector2<f32>) -> Vector2<f32> {
        self.local_to_iso * local
    }

    /// Returns the local translation vector of an isometric translation
    /// vector in this transform's plane. In other words, this converts
    /// isometric coordinates into local coordinates.
    pub fn iso_to_local(&self, iso: Vector2<f32>) -> Vector2<f32> {
        self.iso_to_local * iso
    }

    /// Returns this transform's local translation vector. In other words,
    /// this returns this transform's local coordinates.
    pub fn local(&self) -> Vector2<f32> {
        self.iso_to_local(self.translation)
    }

    /// Changes the angle or scale of this transform.
    /// In other words, if you consider an isometric unit,
    /// the angle parameter is half the angle of the lateral corners.
    pub fn set_isometry(&mut self, angle: Option<Deg<f32>>, scale: Option<Vector2<f32>>) {
        let angle = angle.unwrap_or(self.angle);
        let scale = scale.unwrap_or(self.scale);
        let (local_to_iso, iso_to_local) = matrices_from_angle(angle, scale);
        self.local_to_iso = local_to_iso;
        self.iso_to_local = iso_to_local;
        self.angle = angle;
        self.scale = scale;
    }

    /// Changes the square unit dimensions of this transform.
    /// In other words, if you consider an isometric square unit, this function
    /// will change the isometric grid so that this unit fits in a rectangle of
    /// size (w;h) in the local coordinates system.
    pub fn set_square_unit_dimensions(&mut self, w: f32, h: f32) {
        let angle = Deg((h / w).atan());
        self.set_isometry(Some(angle), Some(Vector2 { x: 1.0, y: 1.0 }));
    }
}
