pub struct NamedPatchDesc {
    /// Named points in the patch
    named_points: Vec<String>,
    expansion_factor: f64,
    render_px_per_model: f64,
}

pub struct NamedPatch {
    /// Named points in the patch
    named_points: Vec<String>,
    expansion_factor: f64,
    render_px_per_model: f64,
    ///
    patch: Option<Box<Patch>>,
}
