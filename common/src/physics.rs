//! Physics processing.

use legion::*;
use num_traits::cast::FromPrimitive;

/// The scalar type used for physics calculations.
/// It's a fixed point type. Computations on an i7 using integers are still just a bit faster
/// than floating point calculations. On top of that, this is very deterministic and portable.
///
/// 53 bits are used for block location and 11 bits are used for sub-block location.
/// This gives the world a free range of +/- 562949953421312 blocks, a freedom of 16384 units
/// within a block, and a minimal velocity of 0.06103515625 blocks/s.
pub type PhysicsScalar = simba::scalar::FixedI50F14;

/// A 2D vector comprised of PhysicsScalars.
// Just not used yet.
// type PhysicsVec2 = nalgebra::Vector2<PhysicsScalar>;

/// used for quick and convenient construction of 3D vectors.
pub trait VectorConstructors3D<T> {
    /// Return the vector but with all the components being set to zero.
    fn zeroed() -> T;

    /// Create a new instance of a vector.
    fn from_i64s(x: i64, y: i64, z: i64) -> Option<T>;

    /// Create a new instance of a vector as a point.
    /// It should be in the center of the block space of the block indexed.
    fn center_of_block(x: i64, y: i64, z: i64) -> Option<T>;

    /// Create a new instance of a vector as a point.
    /// It should be in the center of the block space, but at the bottom of it.
    /// This is nice if you just want to drop an entity on top of a block.
    fn center_bottom_of_block(x: i64, y: i64, z: i64) -> Option<T>;
}

/// A 3D vector comprised of PhysicsScalars.
pub type PhysicsVec3 = nalgebra::Vector3<PhysicsScalar>;

// TODO is there a way to do this without lazy statics?
// It's likely checking every time we access these if they need to be initialized.
lazy_static::lazy_static! {
    static ref VECTOR_ZERO_3D: PhysicsVec3 = PhysicsVec3::new(
        PhysicsScalar::from_f32(0.0).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.0).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.0).expect("Hard coded value incorrect."));
}

lazy_static::lazy_static! {
    static ref BLOCK_CENTER_OFFSET: PhysicsVec3 = PhysicsVec3::new(
        PhysicsScalar::from_f32(0.5).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.5).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.5).expect("Hard coded value incorrect."));
}

lazy_static::lazy_static! {
    static ref BLOCK_CENTER_BOTTOM_OFFSET: PhysicsVec3 = PhysicsVec3::new(
        PhysicsScalar::from_f32(0.5).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.0).expect("Hard coded value incorrect."),
        PhysicsScalar::from_f32(0.5).expect("Hard coded value incorrect."));
}

impl VectorConstructors3D<PhysicsVec3> for PhysicsVec3 {
    fn zeroed() -> Self {
        *VECTOR_ZERO_3D
    }

    fn from_i64s(x: i64, y: i64, z: i64) -> Option<Self> {
        Some(PhysicsVec3::new(PhysicsScalar::from_i64(x)?, PhysicsScalar::from_i64(y)?, PhysicsScalar::from_i64(z)?))
    }

    fn center_of_block(x: i64, y: i64, z: i64) -> Option<Self> {
        Some(
            PhysicsVec3::new(PhysicsScalar::from_i64(x)?, PhysicsScalar::from_i64(y)?, PhysicsScalar::from_i64(z)?)
                + *BLOCK_CENTER_OFFSET,
        )
    }

    fn center_bottom_of_block(x: i64, y: i64, z: i64) -> Option<Self> {
        Some(
            PhysicsVec3::new(PhysicsScalar::from_i64(x)?, PhysicsScalar::from_i64(y)?, PhysicsScalar::from_i64(z)?)
                + *BLOCK_CENTER_BOTTOM_OFFSET,
        )
    }
}

/// Give a physical location aspect to an entity.
///
/// This does not give it the ability to move or have velocity or mass.
/// It does not provide a physical shape.
#[derive(Debug)]
pub struct Positional {
    position: PhysicsVec3,
    angle: PhysicsScalar,
}

impl Positional {
    /// Creates a new Positional component at the specified location with the
    /// specified angle.
    pub fn new(position: PhysicsVec3, angle: PhysicsScalar) -> Positional {
        Positional { position, angle }
    }
}

/// Gives an entity with the Positional component the ability to move.
/// Is useless to any entity without a Positional component.
///
/// Does not provide a physical shape, so it can't collide or interact
/// with other entities until a PhysicalForm component is given.
#[derive(Debug)]
pub struct Movable {
    mass: PhysicsScalar,
    angular_velocity: PhysicsScalar,
    velocity: PhysicsVec3,
}

impl Movable {
    /// Creates a new Movable component with the specified mass, velocity, and angular velocity.
    pub fn new(mass: PhysicsScalar, velocity: PhysicsVec3, angular_velocity: PhysicsScalar) -> Movable {
        Movable { mass, velocity, angular_velocity }
    }
}

/// Gives a simple cylinder physical form to an entity.
#[derive(Debug)]
pub struct CylinderPhysicalForm {
    radius: PhysicsScalar,
    height: PhysicsScalar,
}

impl CylinderPhysicalForm {
    /// Creates a new cylinder physical shape for an entity.
    pub fn new(radius: PhysicsScalar, height: PhysicsScalar) -> CylinderPhysicalForm {
        CylinderPhysicalForm { radius, height }
    }
}

#[derive(Debug)]
struct BoxShape {
    width: PhysicsScalar,
    height: PhysicsScalar,
}

#[derive(Debug)]
struct ComplexBoxShape {
    // TODO this is good information for *building* the shape, not processing it.
    parts: Vec<BoxShape>,
}

/// Gives a complicated physical form to an entity.
///
/// Physical forms have two aspects, the shape and the height.
/// The shape is 2D and made of squares and rectangles. A new vector is
/// allocated for every physical form, so if you make a lot of copies of
/// the same shape, you can expect to see a lot of memory used.
///
/// You should generally prefer the CylinderPhysicalForm.
#[derive(Debug)]
pub struct ComplexPhysicalForm {
    shape: ComplexBoxShape,
    height: PhysicsScalar,
}

#[system(par_for_each)]
fn physics_movement(position: &mut Positional, movement: &Movable) {
    println!("PhysicsMovement: {:?}, {:?}", position, movement);
}

#[system(par_for_each)]
fn cylinder_collision_checking(physical_form: &CylinderPhysicalForm, position: &Positional) {
    println!("CylinderCollisionChecking: {:?}, {:?}", position, physical_form);
}

#[system(par_for_each)]
fn complex_collision_checking(physical_form: &ComplexBoxShape, position: &Positional) {
    println!("ComplexCollisionChecking: {:?}, {:?}", position, physical_form);
}

/// Add systems needed to use the physics engine to the dispatcher builder.
pub fn add_systems(schedule: &mut systems::Builder) {
    schedule
        .add_system(physics_movement_system())
        .flush()
        .add_system(cylinder_collision_checking_system())
        .add_system(complex_collision_checking_system());
}
