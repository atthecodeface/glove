//a Documentation
/*!

# Mesh

This is currently the convex hull.

A mesh is constructed with:

 *  let mut mesh = Mesh::default();

 *  Many mesh.add_pt( Point2D  )

 *  mesh.create_mesh_triangles()

 *  while mesh.optimize_mesh_quads() {}

Or

 * let mesh = Mesh::optimized( my_pts.iter().copied() );

*/
mod line_index;
mod point_index;
mod triangle_index;
pub use line_index::LineIndex;
pub use point_index::PointIndex;
pub use triangle_index::TriangleIndex;

mod index_lt;
pub use index_lt::{IndexLine, IndexTriangle};

mod mesh;
pub use mesh::Mesh;
