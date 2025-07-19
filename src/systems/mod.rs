pub mod data;
pub mod physics;
// pub mod gpu_physics; // Disabled - complex Bevy render API
pub mod optimized_physics;
pub mod collision;
pub mod analytics;
pub mod rendering;
pub mod stress_test;

pub use data::*;
pub use physics::*;
// pub use gpu_physics::*; // Disabled
pub use optimized_physics::*;
pub use collision::*;
pub use analytics::*;
pub use rendering::*;
pub use stress_test::*;