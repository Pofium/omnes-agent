//! Hardware control config (the `[hardware]` section): opt-in access to
//! GPIO, USB-tethered microcontrollers, and SWD/JTAG debug probes, plus the
//! wizard-driven transport mode and serial/baud settings.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{HardwareConfig, HardwareTransport}` are unchanged.

use super::*;

// ── Hardware Config (wizard-driven) ─────────────────────────────

/// Hardware transport mode.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub enum HardwareTransport {
    #[default]
    None,
    Native,
    Serial,
    Probe,
}

impl std::fmt::Display for HardwareTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Native => write!(f, "native"),
            Self::Serial => write!(f, "serial"),
            Self::Probe => write!(f, "probe"),
        }
    }
}

/// Wizard-driven hardware configuration for physical world interaction.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "hardware"]
pub struct HardwareConfig {
    /// Opt in to direct physical-hardware control — GPIO pins, USB-tethered microcontrollers (Arduino, ESP32, Nucleo), or SWD/JTAG debug probes. Leave off for software-only use; turning it on without the right transport configured does nothing.
    #[serde(default)]
    pub enabled: bool,
    /// How OmnesAgent reaches the hardware: `native` = Linux SBC with direct GPIO access (Raspberry Pi, Orange Pi); `serial` = USB-tethered microcontroller speaking over a TTY; `probe` = SWD/JTAG debug probe driving a target chip via probe-rs; `none` = disabled.
    #[serde(default)]
    pub transport: HardwareTransport,
    /// TTY path for the `serial` transport — e.g. `/dev/ttyACM0` on Linux, `/dev/tty.usbmodem1` on macOS, `COM3` on Windows. Ignored for other transports.
    #[serde(default)]
    pub serial_port: Option<String>,
    /// Baud rate negotiated on the serial link. 115200 matches the common Arduino / ESP32 bootloader default; bump to 230400+ when your firmware explicitly supports faster rates and you need the throughput.
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    /// Target chip identifier for `transport = probe` (e.g. `STM32F401RE`, `nRF52840_xxAA`). Passed straight to probe-rs for flash/debug operations; must match a chip probe-rs recognizes.
    #[serde(default)]
    pub probe_target: Option<String>,
    /// Index pre-converted `.md` and `.txt` datasheets from the workspace into
    /// a local RAG store, so the agent can look up pin assignments and
    /// electrical specs inline when you ask hardware questions. PDFs are not
    /// parsed as text; download or convert them to `.md`/`.txt` first. Off by
    /// default — turn on once the workspace has relevant text datasheets.
    #[serde(default)]
    pub workspace_datasheets: bool,
}

fn default_baud_rate() -> u32 {
    115_200
}

impl HardwareConfig {
    /// Return the active transport mode.
    pub fn transport_mode(&self) -> HardwareTransport {
        self.transport.clone()
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transport: HardwareTransport::None,
            serial_port: None,
            baud_rate: default_baud_rate(),
            probe_target: None,
            workspace_datasheets: false,
        }
    }
}
