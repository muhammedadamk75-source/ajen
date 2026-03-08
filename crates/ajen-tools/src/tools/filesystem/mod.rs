mod list_directory;
mod read_file;
mod write_file;

pub use list_directory::ListDirectoryTool;
pub use read_file::ReadFileTool;
pub use write_file::WriteFileTool;

use ajen_core::traits::Tool;
use std::sync::Arc;

pub fn register_all() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ReadFileTool),
        Arc::new(WriteFileTool),
        Arc::new(ListDirectoryTool),
    ]
}
