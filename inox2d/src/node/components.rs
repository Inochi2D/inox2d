/*!
Inochi2D node types to Inox2D components:
- Node -> (Nothing)
- Part -> Drawable + TexturedMesh
- Composite -> Drawable + Composite
- SimplePhysics -> SimplePhysics
- Custom nodes by inheritance -> Custom nodes by composition
*/

pub mod composite;
pub mod drawable;
pub mod simple_physics;
pub mod textured_mesh;

/// Internal component solving for deforms of a node.
pub(crate) mod deform_stack;

pub use composite::Composite;
pub use drawable::Drawable;
pub use simple_physics::SimplePhysics;
pub use textured_mesh::TexturedMesh;

pub(crate) use deform_stack::DeformStack;
