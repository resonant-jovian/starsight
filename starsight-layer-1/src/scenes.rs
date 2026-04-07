//! Scene graph: a tree of drawing primitives backends consume.
//!
//! Status: stub. Implementation lands in 0.5.0+ when the renderer switches from
//! direct backend calls to a recorded scene graph that backends interpret. The
//! scene graph approach enables: spatial indexing for hit-testing, pre-render
//! optimization passes, and identical output across raster and vector backends.

// ── SceneNode ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub enum SceneNode {
//     Path(Path, PathStyle),
//     Text { content: String, position: Point, font_size: f32, color: Color },
//     Group(Vec<SceneNode>),
//     Clip { rect: Rect, child: Box<SceneNode> },
// }

// ── Scene ────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct Scene {
//     root: SceneNode,
//     bounds: Rect,
// }

// ── Scene::render ────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): impl Scene { pub fn render(&self, backend: &mut dyn DrawBackend) -> Result<()> { ... } }
