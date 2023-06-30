#[cfg(target_os = "windows")]
use embed_manifest::{
    embed_manifest,
    manifest::{
        ActiveCodePage, DpiAwareness, ExecutionLevel, HeapType, MaxVersionTested,
        ScrollingAwareness, Setting, SupportedOS,
    },
    new_manifest,
};

fn main() {
    #[cfg(target_os = "windows")]
    {
        let manifest = new_manifest("QR-Code-Generierer")
            .version(0, 0, 1, 1)
            .max_version_tested(MaxVersionTested::Windows10Version2004)
            .supported_os(SupportedOS::Windows10..)
            .active_code_page(ActiveCodePage::Utf8)
            .dpi_awareness(DpiAwareness::PerMonitor) // TODO
            .heap_type(HeapType::SegmentHeap)
            .long_path_aware(Setting::Enabled)
            .scrolling_awareness(ScrollingAwareness::LowResolution)
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .ui_access(false);
        embed_manifest(manifest).expect("unable to embed manifest file");
    }
}
