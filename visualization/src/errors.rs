//! Error types for range bar visualization

use thiserror::Error;

/// Result type alias for visualization operations
pub type Result<T> = std::result::Result<T, VisualizationError>;

/// Errors that can occur during visualization
#[derive(Error, Debug)]
pub enum VisualizationError {
    #[error("Invalid data format: {message}")]
    InvalidData { message: String },
    
    #[error("Chart rendering failed: {message}")]
    RenderingError { message: String },
    
    #[error("File I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    
    #[error("Image processing error: {message}")]
    ImageError { message: String },
    
    #[error("Layout calculation error: {message}")]
    LayoutError { message: String },
    
    #[error("Data preprocessing error: {message}")]
    PreprocessingError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

impl From<image::ImageError> for VisualizationError {
    fn from(err: image::ImageError) -> Self {
        VisualizationError::ImageError {
            message: err.to_string(),
        }
    }
}

impl<T: std::error::Error + Send + Sync + 'static> From<plotters::drawing::DrawingAreaErrorKind<T>> for VisualizationError {
    fn from(err: plotters::drawing::DrawingAreaErrorKind<T>) -> Self {
        VisualizationError::RenderingError {
            message: format!("Drawing area error: {}", err),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for VisualizationError {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        VisualizationError::RenderingError {
            message: format!("Boxed error: {}", err),
        }
    }
}

impl From<Box<dyn std::error::Error + 'static>> for VisualizationError {
    fn from(err: Box<dyn std::error::Error + 'static>) -> Self {
        VisualizationError::RenderingError {
            message: format!("Boxed error: {}", err),
        }
    }
}

// Temporarily commented out for polars compatibility
// impl From<polars::PolarsError> for VisualizationError {
//     fn from(err: polars::PolarsError) -> Self {
//         VisualizationError::PreprocessingError {
//             message: err.to_string(),
//         }
//     }
// }