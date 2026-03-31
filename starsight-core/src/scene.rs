//!

///
use crate::backend::{DrawBackend, Path};
///
use crate::error::Result;
///
use crate::style::Style;
///
use crate::geom::Rect;
///
use crate::geom::TextBlock;
///
use starsight_marks::position::{Position, Transform};

///
pub enum SceneNode {
    ///
    Path { path: Path, style: Style },
    ///
    Text {
        block: TextBlock,
        position: Position,
    },
    ///
    Group {
        children: Vec<SceneNode>,
        transform: Transform,
    },
    ///
    Clip { rect: Rect, child: Box<SceneNode> },
}

///
pub struct Scene {
    ///
    root: Vec<SceneNode>,
}
///
impl Scene {
    ///
    pub fn render(&self, backend: &mut dyn DrawBackend) -> Result<()> {
        todo!()
    }
}
