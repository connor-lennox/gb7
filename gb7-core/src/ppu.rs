pub struct Ppu {
    mode: PpuMode,
    line_cycles: u32,
    reached_window: bool,
    window_line_counter: u16,
}

enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}