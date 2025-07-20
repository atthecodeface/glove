const π: f64 = std::f64::consts::PI;
fn plot_of_theta(θ: f64) -> impl poloto::build::PlotIterator<L = (f64, f64)> {
    let μs = (0..=200).map(move |x| -1.0 + (x as f64) / 100.0);
    let sθ = θ.sin();
    let r = move |μ: f64| {
        let α = μ * π / 2.0;
        ///   ρ = 2.cos(θ-α).cos(α)/sin(θ)  ;  k = 1 - 2cos(θ-α).sin(α)/sin(θ)
        let θ_m_α = θ - α;
        let c_θ_m_α = θ_m_α.cos().max(0.0);
        let c = c_θ_m_α / sθ * 2.0;
        [1.0 - c * α.sin(), c * α.cos()]
    };

    let C = (π / 2.0 - θ).tan();
    let R = 1.0 / θ.sin();
    let r = move |μ: f64| {
        let γ = μ * (π - θ);
        [R * γ.sin(), C + R * γ.cos()]
    };
    let plot = poloto::build::plot(format!("θ={:0.1}", θ / π * 180.0));
    plot.line(μs.map(r))
}

pub fn main() -> Result<(), String> {
    use poloto::build::PlotIterator;
    let plots = poloto::build::origin();

    let w = 16.0;
    let h = w * 0.6;
    let bg: &[[f64; 2]] = &[
        [-1.0, 0.],
        [1.0, 0.],
        [w / 2.0, 0.],
        [w / 2.0, h],
        [-w / 2.0, h],
        [-w / 2.0, 0.0],
    ];
    let plot = poloto::build::plot("Background");
    let plot = plot.line(bg.into_iter().copied());
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.07 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.08 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.09 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.10 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.125 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.18 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.25 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.375 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.50 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.625 * π);
    let plots = plots.chain(plot);

    let plot = plot_of_theta(0.75 * π);
    let plots = plots.chain(plot);

    poloto::frame_build()
        .data(plots)
        .build_and_label(("Circles", "x", "y"))
        .append_to(poloto::header().light_theme())
        .render_stdout();

    Ok(())
}
