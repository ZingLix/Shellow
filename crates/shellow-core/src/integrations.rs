use crate::IntegrationReport;

pub fn summary_line(report: &IntegrationReport) -> String {
    format!(
        "ghostty={}->{}/{}  libghostty-link={} ready={} abi={}  russh={}  wgpu={}->{} surface={}",
        status(report.ghostty_ready),
        report.terminal_target_backend,
        report.terminal_backend_migration,
        status(report.libghostty_vt_link_configured),
        status(report.libghostty_vt_ready),
        report.libghostty_vt_abi_contract,
        status(report.russh_ready),
        status(report.wgpu_ready),
        report.renderer_target_backend,
        status(report.renderer_surface_ready)
    )
}

fn status(ready: bool) -> &'static str {
    if ready { "ready" } else { "adapter" }
}
