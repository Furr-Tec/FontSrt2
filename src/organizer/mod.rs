//! Font organization and processing functionality

pub mod processor;
pub mod batch;
pub mod group;

pub use processor::organize_fonts;
pub use batch::batch_process;
pub use group::group_by_foundry;

