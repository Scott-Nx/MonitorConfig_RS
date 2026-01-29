use crate::{MonitorError, Result};
use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::HANDLE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcpFeatureResponse {
    pub vcp_code: u8,
    pub current_value: u32,
    pub maximum_value: u32,
    pub code_type: VcpCodeType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VcpCodeType {
    SetParameter = 0,
    Momentary = 1,
}

// Common VCP codes
pub mod codes {
    pub const BRIGHTNESS: u8 = 0x10;
    pub const CONTRAST: u8 = 0x12;
    pub const COLOR_TEMPERATURE: u8 = 0x14;
    pub const RED_GAIN: u8 = 0x16;
    pub const GREEN_GAIN: u8 = 0x18;
    pub const BLUE_GAIN: u8 = 0x1A;
    pub const POWER_MODE: u8 = 0xD6;
    pub const INPUT_SOURCE: u8 = 0x60;
    pub const AUDIO_VOLUME: u8 = 0x62;
    pub const AUDIO_MUTE: u8 = 0x8D;
}

pub struct VcpMonitor {
    handle: HANDLE,
}

impl VcpMonitor {
    pub fn new(handle: HANDLE) -> Self {
        Self { handle }
    }

    pub fn get_vcp_feature(&self, vcp_code: u8) -> Result<VcpFeatureResponse> {
        unsafe {
            let mut code_type = 0u32;
            let mut current_value = 0u32;
            let mut maximum_value = 0u32;

            let result = crate::native::dxva2::GetVCPFeatureAndVCPFeatureReply(
                self.handle,
                vcp_code,
                &mut code_type,
                &mut current_value,
                &mut maximum_value,
            );

            if result == 0 {
                return Err(MonitorError::VcpNotSupported);
            }

            Ok(VcpFeatureResponse {
                vcp_code,
                current_value,
                maximum_value,
                code_type: if code_type == 0 {
                    VcpCodeType::SetParameter
                } else {
                    VcpCodeType::Momentary
                },
            })
        }
    }

    pub fn set_vcp_feature(&self, vcp_code: u8, value: u32) -> Result<()> {
        unsafe {
            let result = crate::native::dxva2::SetVCPFeature(self.handle, vcp_code, value);

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "SetVCPFeature failed".to_string(),
                ));
            }

            Ok(())
        }
    }

    /// Scan all VCP codes (0x00-0xFF) and return the ones supported by the monitor
    /// Similar to PowerShell's Get-MonitorVCPResponse -All
    pub fn scan_vcp_features(&self) -> Vec<VcpFeatureResponse> {
        let mut features = Vec::new();
        
        for code in 0u8..=255u8 {
            if let Ok(response) = self.get_vcp_feature(code) {
                features.push(response);
            }
            // Silently ignore unsupported codes (similar to PowerShell behavior)
        }
        
        features
    }

    pub fn get_capabilities(&self) -> Result<String> {
        unsafe {
            let mut length = 0u32;
            let result =
                crate::native::dxva2::GetCapabilitiesStringLength(self.handle, &mut length);

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "GetCapabilitiesStringLength failed".to_string(),
                ));
            }

            let mut buffer = vec![0u8; length as usize];
            let result = crate::native::dxva2::CapabilitiesRequestAndCapabilitiesReply(
                self.handle,
                buffer.as_mut_ptr(),
                length,
            );

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "CapabilitiesRequestAndCapabilitiesReply failed".to_string(),
                ));
            }

            // Remove null terminators and convert to String
            let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
            Ok(String::from_utf8_lossy(&buffer[..end]).to_string())
        }
    }

    pub fn save_settings(&self) -> Result<()> {
        unsafe {
            let result = crate::native::dxva2::SaveCurrentMonitorSettings(self.handle);

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "SaveCurrentMonitorSettings failed".to_string(),
                ));
            }

            Ok(())
        }
    }

    pub fn restore_factory_defaults(&self) -> Result<()> {
        unsafe {
            let result = crate::native::dxva2::RestoreMonitorFactoryDefaults(self.handle);

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "RestoreMonitorFactoryDefaults failed".to_string(),
                ));
            }

            Ok(())
        }
    }

    pub fn restore_factory_color_defaults(&self) -> Result<()> {
        unsafe {
            let result = crate::native::dxva2::RestoreMonitorFactoryColorDefaults(self.handle);

            if result == 0 {
                return Err(crate::MonitorError::UnsupportedOperation(
                    "RestoreMonitorFactoryColorDefaults failed".to_string(),
                ));
            }

            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VcpFeatureInfo {
    pub code: u8,
    pub name: &'static str,
    pub description: &'static str,
}

pub const KNOWN_VCP_CODES: &[VcpFeatureInfo] = &[
    VcpFeatureInfo {
        code: codes::BRIGHTNESS,
        name: "Brightness",
        description: "Luminance of the image (Brightness control)",
    },
    VcpFeatureInfo {
        code: codes::CONTRAST,
        name: "Contrast",
        description: "Contrast of the image",
    },
    VcpFeatureInfo {
        code: codes::COLOR_TEMPERATURE,
        name: "Color Temperature",
        description: "Select a specified color temperature",
    },
    VcpFeatureInfo {
        code: codes::RED_GAIN,
        name: "Red Video Gain",
        description: "Red video gain (drive)",
    },
    VcpFeatureInfo {
        code: codes::GREEN_GAIN,
        name: "Green Video Gain",
        description: "Green video gain (drive)",
    },
    VcpFeatureInfo {
        code: codes::BLUE_GAIN,
        name: "Blue Video Gain",
        description: "Blue video gain (drive)",
    },
    VcpFeatureInfo {
        code: codes::POWER_MODE,
        name: "Power Mode",
        description: "DPM and DPMS status (1=On, 4=Off)",
    },
    VcpFeatureInfo {
        code: codes::INPUT_SOURCE,
        name: "Input Source",
        description: "Select input source",
    },
    VcpFeatureInfo {
        code: codes::AUDIO_VOLUME,
        name: "Audio Speaker Volume",
        description: "Audio speaker volume",
    },
    VcpFeatureInfo {
        code: codes::AUDIO_MUTE,
        name: "Audio Mute",
        description: "Audio mute/unmute",
    },
];

pub fn get_vcp_code_info(code: u8) -> Option<&'static VcpFeatureInfo> {
    KNOWN_VCP_CODES.iter().find(|info| info.code == code)
}
