use crate::error::AppError;
use crate::tools::atlaspro::model::{
    AtlasProExecuteRequest, AtlasProReport, AtlasProScanRequest, SpriteSource,
};

#[tauri::command]
pub fn scan_atlaspro_inputs(request: AtlasProScanRequest) -> Result<Vec<SpriteSource>, AppError> {
    crate::tools::atlaspro::scanner::scan(request)
}

#[tauri::command]
pub fn execute_atlaspro(request: AtlasProExecuteRequest) -> Result<AtlasProReport, AppError> {
    crate::tools::atlaspro::pipeline::execute(request)
}
