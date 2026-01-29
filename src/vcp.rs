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
        code: 0x00,
        name: "Code Page",
        description: "Returns the Code Page ID number Byte SL.",
    },
    VcpFeatureInfo {
        code: 0x01,
        name: "Degauss",
        description: "Causes a CRT display to perform a degauss cycle.",
    },
    VcpFeatureInfo {
        code: 0x02,
        name: "New Control Value",
        description: "Indicates that a displays MCCS VCP Code register value has changed.",
    },
    VcpFeatureInfo {
        code: 0x03,
        name: "Soft Controls",
        description: "Allows applications running on the host to use control buttons on the display.",
    },
    VcpFeatureInfo {
        code: 0x04,
        name: "Restore Factory Defaults",
        description: "Restore all factory presets including luminance / contrast, geometry, color and TV defaults.",
    },
    VcpFeatureInfo {
        code: 0x05,
        name: "Restore Factory Luminance / Contrast Defaults",
        description: "Restores factory defaults for luminance and contrast adjustments.",
    },
    VcpFeatureInfo {
        code: 0x06,
        name: "Restore Factory Geometry Defaults",
        description: "Restore factory defaults for geometry adjustments.",
    },
    VcpFeatureInfo {
        code: 0x08,
        name: "Restore Factory Color Defaults",
        description: "Restore factory defaults for color settings.",
    },
    VcpFeatureInfo {
        code: 0x0A,
        name: "Restore Factory TV Defaults",
        description: "Restore factory defaults for TV functions.",
    },
    VcpFeatureInfo {
        code: 0x0B,
        name: "User Color Temperature Increment",
        description: "Sets the minimum increment in which the display can adjust the color temperature.",
    },
    VcpFeatureInfo {
        code: 0x0C,
        name: "User Color Temperature",
        description: "Multiplier of the value set in 0x0B",
    },
    VcpFeatureInfo {
        code: 0x0E,
        name: "Clock",
        description: "Video sampling clock frequency",
    },
    VcpFeatureInfo {
        code: codes::BRIGHTNESS,
        name: "Luminance",
        description: "Luminance of the image (Brightness control).",
    },
    VcpFeatureInfo {
        code: 0x11,
        name: "Flesh Tone Enhancement",
        description: "This control allows for selection of contrast enhancement algorithms using a bitmask.",
    },
    VcpFeatureInfo {
        code: codes::CONTRAST,
        name: "Contrast",
        description: "Contrast of the image.",
    },
    VcpFeatureInfo {
        code: 0x13,
        name: "Backlight Control",
        description: "This VCP code has been deprecated.",
    },
    VcpFeatureInfo {
        code: codes::COLOR_TEMPERATURE,
        name: "Select Color Preset",
        description: "Select a specified color temperature.",
    },
    VcpFeatureInfo {
        code: codes::RED_GAIN,
        name: "Video Gain (Drive): Red",
        description: "Sets the luminance of red pixels.",
    },
    VcpFeatureInfo {
        code: 0x17,
        name: "User Color Vision Compensation",
        description: "Sets the degree of compensation. Intended to help people that see red colors poorly.",
    },
    VcpFeatureInfo {
        code: codes::GREEN_GAIN,
        name: "Video Gain (Drive): Green",
        description: "Sets the luminance of green pixels.",
    },
    VcpFeatureInfo {
        code: codes::BLUE_GAIN,
        name: "Video Gain (Drive): Blue",
        description: "Sets the luminance of blue pixels.",
    },
    VcpFeatureInfo {
        code: 0x1C,
        name: "Focus",
        description: "Sets the focus of the image.",
    },
    VcpFeatureInfo {
        code: 0x1E,
        name: "Auto Setup",
        description: "Perform auto setup function (H/V position, clock, clock phase, A/D converter, etc.)",
    },
    VcpFeatureInfo {
        code: 0x1F,
        name: "Auto Color Setup",
        description: "Perform auto color setup function (R / G / B gain and offset, A/D setup, etc.)",
    },
    VcpFeatureInfo {
        code: 0x20,
        name: "Horizontal Position (Phase)",
        description: "Moves the image left and right on the display.",
    },
    VcpFeatureInfo {
        code: 0x22,
        name: "Horizontal Size",
        description: "Sets the width of the image.",
    },
    VcpFeatureInfo {
        code: 0x24,
        name: "Horizontal Pincushion",
        description: "Makes the left/right sides of the image more/less convex.",
    },
    VcpFeatureInfo {
        code: 0x26,
        name: "Horizontal Pincushion Balance",
        description: "Increasing (decreasing) this value will move the center section of the image toward the right (left) side of the display.",
    },
    VcpFeatureInfo {
        code: 0x28,
        name: "Horizontal Convergence R/B",
        description: "Increasing (decreasing) this value will shift the red pixels to the right (left) across the image and the blue pixels left (right) across the image with respect to the green pixels.",
    },
    VcpFeatureInfo {
        code: 0x29,
        name: "Horizontal Convergence M/G",
        description: "Increasing (decreasing) this value will shift the magenta pixels to the right (left) across the image and the green pixels left (right) across the image with respect to the magenta pixels",
    },
    VcpFeatureInfo {
        code: 0x2A,
        name: "Horizontal Linearity",
        description: "Increasing (decreasing) this value will increase (decrease) the density of pixels in the image center",
    },
    VcpFeatureInfo {
        code: 0x2C,
        name: "Horizontal Linearity Balance",
        description: "Increasing (decreasing) this value shifts the density of pixels from the left (right) side to the right (left) side of the image.",
    },
    VcpFeatureInfo {
        code: 0x2E,
        name: "Gray Scale Expansion",
        description: "Expands the gray scale either in the near white region or the near black region (or both).",
    },
    VcpFeatureInfo {
        code: 0x30,
        name: "Vertical Position (Phase)",
        description: "Increasing (decreasing) this value moves the image toward the top (bottom) edge of the display.",
    },
    VcpFeatureInfo {
        code: 0x32,
        name: "Vertical Size",
        description: "Increasing (decreasing) this value will increase (decrease) the height of the image",
    },
    VcpFeatureInfo {
        code: 0x34,
        name: "Vertical Pincushion",
        description: "Increasing (decreasing) this value will cause the top and bottom edges of the image to become more (less) convex.",
    },
    VcpFeatureInfo {
        code: 0x36,
        name: "Vertical Pincushion Balance",
        description: "Increasing (decreasing) this value will move the center section of the image toward the top (bottom) edge of the display.",
    },
    VcpFeatureInfo {
        code: 0x38,
        name: "Vertical Convergence R/B",
        description: "Increasing (decreasing) this value shifts the red pixels up (down) across the image and the blue pixels down (up) across the image with respect to the green pixels.",
    },
    VcpFeatureInfo {
        code: 0x39,
        name: "Vertical Convergence M/G",
        description: "Increasing (decreasing) this value will shift the magenta pixels up (down) across the image and the green pixels down (up) across the image with respect to the magenta pixels",
    },
    VcpFeatureInfo {
        code: 0x3A,
        name: "Vertical Linearity",
        description: "Increasing (decreasing) this value will increase (decrease) the density of scan lines in the image center.",
    },
    VcpFeatureInfo {
        code: 0x3C,
        name: "Vertical Linearity Balance",
        description: "Increasing (decreasing) this value shifts the density of scan lines from the top (bottom) end to the bottom (top) end of the image",
    },
    VcpFeatureInfo {
        code: 0x3E,
        name: "Clock Phase",
        description: "Increasing (decreasing) this value will increase (decrease) the phase shift of the sampling clock.",
    },
    VcpFeatureInfo {
        code: 0x40,
        name: "Horizontal Parallelogram",
        description: "Increasing (decreasing) this value shifts the top section of the image to the right (left) with respect to the bottom section of the image",
    },
    VcpFeatureInfo {
        code: 0x41,
        name: "Vertical Parallelogram",
        description: "Increasing (decreasing) this value shifts the top section of the image to the right (left) with respect to the bottom section of the image.",
    },
    VcpFeatureInfo {
        code: 0x42,
        name: "Horizontal Keystone",
        description: "Increasing (decreasing) this value will increase (decrease) the horizontal size at the top of the image with respect to the horizontal size at the bottom of the image",
    },
    VcpFeatureInfo {
        code: 0x43,
        name: "Vertical Keystone",
        description: "Increasing (decreasing) this value will increase (decrease) the vertical size at the left of the image with respect to the vertical size at the right of the image",
    },
    VcpFeatureInfo {
        code: 0x44,
        name: "Rotation",
        description: "Increasing (decreasing) this value rotates the image (counter) clockwise about the center point of the screen.",
    },
    VcpFeatureInfo {
        code: 0x46,
        name: "Top Corner Flare",
        description: "Increasing (decreasing) this value will increase (decrease) the distance between the left and right sides at the top of the image.",
    },
    VcpFeatureInfo {
        code: 0x48,
        name: "Top Corner Hook",
        description: "Increasing (decreasing) this value moves the top of the image to the right (left).",
    },
    VcpFeatureInfo {
        code: 0x4A,
        name: "Bottom Corner Flare",
        description: "Increasing (decreasing) this value will increase (decrease) the distance between the left and right sides at the bottom of the image",
    },
    VcpFeatureInfo {
        code: 0x4C,
        name: "Bottom Corner Hook",
        description: "Increasing (decreasing) this value moves the bottom of the image to the right (left).",
    },
    VcpFeatureInfo {
        code: 0x52,
        name: "Active Control",
        description: "All VCP Codes that have new values must be added to this FIFO in the order they occur and VCP 02h must be set to = 02h when this FIFO is NOT empty.",
    },
    VcpFeatureInfo {
        code: 0x54,
        name: "Performance Preservation",
        description: "This command provides the capability to control up to 16 features aimed at maintaining the performance of a display.",
    },
    VcpFeatureInfo {
        code: 0x56,
        name: "Horizontal Moir",
        description: "Increasing (decreasing) this value controls the horizontal picture moir cancellation.",
    },
    VcpFeatureInfo {
        code: 0x58,
        name: "Vertical Moir",
        description: "Increasing (decreasing) this value controls the vertical picture moir cancellation.",
    },
    VcpFeatureInfo {
        code: 0x59,
        name: "6 Axis Saturation Control: Red",
        description: "Adjust the red saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: 0x5A,
        name: "6 Axis Saturation Control: Yellow",
        description: "Adjust the yellow saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: 0x5B,
        name: "6 Axis Saturation Control: Green",
        description: "Adjust the green saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: 0x5C,
        name: "6 Axis Saturation Control: Cyan",
        description: "Adjust the cyan saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: 0x5D,
        name: "6 Axis Saturation Control: Blue",
        description: "Adjust the blue saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: 0x5E,
        name: "6 Axis Saturation Control: Magenta",
        description: "Adjust the magenta saturation for 6-axis color.",
    },
    VcpFeatureInfo {
        code: codes::INPUT_SOURCE,
        name: "Input Select",
        description: "Adjusts the active input on the display.",
    },
    VcpFeatureInfo {
        code: codes::AUDIO_VOLUME,
        name: "Audio: Speaker Volume",
        description: "Allows the volume to be adjusted.",
    },
    VcpFeatureInfo {
        code: 0x63,
        name: "Speaker Select",
        description: "Selects the active speakers on the display",
    },
    VcpFeatureInfo {
        code: 0x64,
        name: "Audio: Microphone Volume",
        description: "Sets the microphone gain.",
    },
    VcpFeatureInfo {
        code: 0x65,
        name: "Audio: Jack Connection Status",
        description: "This bitmask allows the source to determine the capabilities as well as the current configuration of speakers/lineout connected to a display, or active in an audio only device",
    },
    VcpFeatureInfo {
        code: 0x66,
        name: "Ambient Light Sensor",
        description: "Used to control the action of an ambient light sensor",
    },
    VcpFeatureInfo {
        code: 0x6B,
        name: "Backlight Level: White",
        description: "Sets the White backlight level of the image.",
    },
    VcpFeatureInfo {
        code: 0x6C,
        name: "Video Black Level: Red",
        description: "Sets the black level of the red video.",
    },
    VcpFeatureInfo {
        code: 0x6D,
        name: "Backlight Level: Red",
        description: "Sets the Red backlight level of the image.",
    },
    VcpFeatureInfo {
        code: 0x6E,
        name: "Video Black Level: Green",
        description: "Sets the black level of the green video.",
    },
    VcpFeatureInfo {
        code: 0x6F,
        name: "Backlight Level: Green",
        description: "Sets the Green backlight level of the image.",
    },
    VcpFeatureInfo {
        code: 0x70,
        name: "Video Black Level: Blue",
        description: "Sets the black level of the blue video",
    },
    VcpFeatureInfo {
        code: 0x71,
        name: "Backlight Level: Blue",
        description: "Sets the Blue backlight level of the image.",
    },
    VcpFeatureInfo {
        code: 0x72,
        name: "Gamma",
        description: "This VCP code has two distinct modes, it may be used to select an absolute (within a defined tolerance) value for gamma, or it may be used to select a value of gamma relative to the default gamma of the display",
    },
    VcpFeatureInfo {
        code: 0x73,
        name: "LUT Size",
        description: "Provides the size (number of entries and number of bits / entry) for the Red / Green and Blue LUT in the display",
    },
    VcpFeatureInfo {
        code: 0x74,
        name: "Single Point LUT Operation",
        description: "Allows a single point within a displays color LUT (look up table) to be loaded.",
    },
    VcpFeatureInfo {
        code: 0x75,
        name: "Block LUT Operation",
        description: "Provides an efficient method for loading multiple values into a displays LUT.",
    },
    VcpFeatureInfo {
        code: 0x76,
        name: "Remote Procedure Call",
        description: "Allows initiation of a routine / macro resident in the display.",
    },
    VcpFeatureInfo {
        code: 0x78,
        name: "Display Identification on Data Operation",
        description: "This command allows a selected block (128 bytes) of Display Identification Data (e.g., EDID or DisplayID) to be read.",
    },
    VcpFeatureInfo {
        code: 0x7C,
        name: "Adjust Zoom",
        description: "Sets the zoom function of the projection lens.",
    },
    VcpFeatureInfo {
        code: 0x82,
        name: "Horizontal Mirror (Flip)",
        description: "This VCP code allows the image to be mirrored horizontally.",
    },
    VcpFeatureInfo {
        code: 0x84,
        name: "Vertical Mirror (Flip)",
        description: "This VCP code allows the image to be mirrored vertically.",
    },
    VcpFeatureInfo {
        code: 0x86,
        name: "Display Scaling",
        description: "Changing this value will affect the scaling (input versus output) function of the display. NOTE: This VCP code can be used to scale up or down to the maximum screen size.",
    },
    VcpFeatureInfo {
        code: 0x87,
        name: "Sharpness",
        description: "Allows one of a range of algorithms to be selected to suit the type of image being displayed and/or personal preference.",
    },
    VcpFeatureInfo {
        code: 0x88,
        name: "Velocity Scan Modulation",
        description: "Increasing (decreasing) this value will increase (decrease) the velocity modulation of the horizontal scan as a function of a change in the luminance level.",
    },
    VcpFeatureInfo {
        code: 0x8A,
        name: "Color Saturation",
        description: "Increasing this control increases the amplitude of the color difference components of the video signal.",
    },
    VcpFeatureInfo {
        code: 0x8B,
        name: "TVChannel Up / Down",
        description: "Used to increment / decrement between TV-channels, the exact behavior is implementation specific (e.g. increment / decrement to next numeric channel or increment / decrement to next channel with a signal).",
    },
    VcpFeatureInfo {
        code: 0x8C,
        name: "TV-Sharpness",
        description: "Increasing this control increases the amplitude of the high frequency components of the video signal.",
    },
    VcpFeatureInfo {
        code: codes::AUDIO_MUTE,
        name: "Audio Mute / Screen Blank",
        description: "Provides for the audio to be muted or un-muted.",
    },
    VcpFeatureInfo {
        code: 0x8E,
        name: "TV-Contrast",
        description: "Increasing (decreasing) this control increases (decreases) the ratio between whites and blacks in the video.",
    },
    VcpFeatureInfo {
        code: 0x8F,
        name: "Audio Treble",
        description: "Allows control of the high frequency component of the audio.",
    },
    VcpFeatureInfo {
        code: 0x90,
        name: "Hue",
        description: "Also known as tint Increasing (decreasing) this control increases (decreases) the wavelength of the color component of the video signal.",
    },
    VcpFeatureInfo {
        code: 0x91,
        name: "Audio Bass",
        description: "Allows control of the low frequency component of the audio.",
    },
    VcpFeatureInfo {
        code: 0x92,
        name: "TV-Black Level / Luminance",
        description: "Increasing this control increases the black level of the video, resulting in an increase of the luminance level of the video.",
    },
    VcpFeatureInfo {
        code: 0x93,
        name: "Audio Balance L / R",
        description: "This control affects the left right balance of audio output. Increasing (decreasing) the value will cause the balance to move to the right (left).",
    },
    VcpFeatureInfo {
        code: 0x94,
        name: "Audio Processor Mode",
        description: "This control allows one of several audio processing modes to be selected.",
    },
    VcpFeatureInfo {
        code: 0x95,
        name: "Window Position (TL_X)",
        description: "Defines the top left X pixel of an area of the image. Specified in coordinates of incoming image before any scaling etc. in the display.",
    },
    VcpFeatureInfo {
        code: 0x96,
        name: "Window Position (TL_Y)",
        description: "Defines the top left Y pixel of an area of the image. Specified in coordinates of incoming image before any scaling etc. in the display.",
    },
    VcpFeatureInfo {
        code: 0x97,
        name: "Window Position (BR_X)",
        description: "Defines the bottom right X pixel of an area of the image. Specified in co-ordinates of the incoming image before any scaling etc. in the display.",
    },
    VcpFeatureInfo {
        code: 0x98,
        name: "Window Position (BR_Y)",
        description: "Defines the bottom right Y pixel of an area of the image. Specified in co-ordinates of the incoming image before any processing (e.g. scaling) in the display",
    },
    VcpFeatureInfo {
        code: 0x9A,
        name: "Window Background",
        description: "Changes the contrast ratio between the area of the window and the rest of the desktop",
    },
    VcpFeatureInfo {
        code: 0x9B,
        name: "6 Axis Hue Control: Red",
        description: "Adjust the red hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0x9C,
        name: "6 Axis Hue Control: Yellow",
        description: "Adjust the yellow hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0x9D,
        name: "6 Axis Hue Control: Green",
        description: "Adjust the green hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0x9E,
        name: "6 Axis Hue Control: Cyan",
        description: "Adjust the cyan hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0x9F,
        name: "6 Axis Hue Control: Blue",
        description: "Adjust the blue hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0xA0,
        name: "6 Axis Hue Control: Magenta",
        description: "Adjust the magenta hue for 6-axis color",
    },
    VcpFeatureInfo {
        code: 0xA2,
        name: "Auto Setup On / Off",
        description: "Turn on / off the auto setup function (periodic or event driven)",
    },
    VcpFeatureInfo {
        code: 0xA4,
        name: "Window Mask Control",
        description: "Data size: Write / Read = 10 bytes This code has two sets of functions:",
    },
    VcpFeatureInfo {
        code: 0xA5,
        name: "Window Select",
        description: "Change the selected window (as defined by 95h 98h).",
    },
    VcpFeatureInfo {
        code: 0xA6,
        name: "Window Size",
        description: "Increasing (decreasing) this value will increase (decrease) the size of the window called out by VCP A5",
    },
    VcpFeatureInfo {
        code: 0xA7,
        name: "Window Transparency",
        description: "Increasing (decreasing) this value will increase (decrease) the transparency of the window called out by A5",
    },
    VcpFeatureInfo {
        code: 0xAA,
        name: "Screen Orientation",
        description: "Indicates the orientation of the screen",
    },
    VcpFeatureInfo {
        code: 0xAC,
        name: "Horizontal Frequency",
        description: "Horizontal synchronization signal frequency in Hz as determined by the display.",
    },
    VcpFeatureInfo {
        code: 0xAE,
        name: "Vertical Frequency",
        description: "Vertical synchronization signal frequency in 0.01Hz as determined by the display",
    },
    VcpFeatureInfo {
        code: 0xB0,
        name: "Settings",
        description: "Store/Restore the user saved values for current mode",
    },
    VcpFeatureInfo {
        code: 0xB2,
        name: "Flat Panel Sub-Pixel Layout",
        description: "Indicates the type of LCD sub-pixel structure",
    },
    VcpFeatureInfo {
        code: 0xB4,
        name: "Source Timing Mode",
        description: "Indicates the timing mode being sent by the host.",
    },
    VcpFeatureInfo {
        code: 0xB5,
        name: "Source Color Coding",
        description: "Allows the host to specify the color coding method that is being used.",
    },
    VcpFeatureInfo {
        code: 0xB6,
        name: "Display Technology Type",
        description: "Indicates the base technology type.",
    },
    VcpFeatureInfo {
        code: 0xB7,
        name: "Monitor Status",
        description: "Video mode and status of a DPVL capable monitor.",
    },
    VcpFeatureInfo {
        code: 0xB8,
        name: "Packet Count",
        description: "Counter for the DPVL packets received (valid and invalid ones). This value counts from 00 00h to FF FFh and then rolls over to 00 00h. The host can reset the value to 00 00h",
    },
    VcpFeatureInfo {
        code: 0xB9,
        name: "Monitor X Origin",
        description: "The X origin of the monitor in the virtual screen. The support of this command indicates the multi-display support of the display. If a display supports this command, the monitor must also support Monitor Y Origin command",
    },
    VcpFeatureInfo {
        code: 0xBA,
        name: "Monitor Y Origin",
        description: "The Y origin of the display in the virtual screen. The support of this command indicates the multi-display support of the display. If a display supports this command, the monitor must also support Monitor X Origin command",
    },
    VcpFeatureInfo {
        code: 0xBB,
        name: "Header Error Count",
        description: "Error Counter for the DPVL header. The counter value saturates at FF FFh. Host can reset to 00 00h.",
    },
    VcpFeatureInfo {
        code: 0xBC,
        name: "Body CRC Error Count",
        description: "CRC error Counter for the DPVL body (containing video data). The counter value saturates at FF FFh. The Host can reset to 00 00h",
    },
    VcpFeatureInfo {
        code: 0xBD,
        name: "Client ID",
        description: "Assigned identification number for the monitor. Valid range is 0000h to FF FEh FF FFh is reserved for broadcast.",
    },
    VcpFeatureInfo {
        code: 0xBE,
        name: "Link Control",
        description: "Indicates the status of the DVI link",
    },
    VcpFeatureInfo {
        code: 0xC0,
        name: "Display Usage Time",
        description: "Returns the current value (in hours) of active power on time accumulated by the display in the ML, SH and SL bytes",
    },
    VcpFeatureInfo {
        code: 0xC2,
        name: "Display Descriptor Length",
        description: "Returns the length (in bytes) of non-volatile storage in the display available for writing a display descriptor the maximum descriptor length is 256 bytes",
    },
    VcpFeatureInfo {
        code: 0xC3,
        name: "Transmit Display Descriptor",
        description: "Allows a display descriptor (up to maximum length defined by the display (see code C2h) to be written (read) to (from) nonvolatile storage in the display.",
    },
    VcpFeatureInfo {
        code: 0xC4,
        name: "Enable Display of Display Descriptor",
        description: "If enabled, the display descriptor written to the display using VCP code C3h must be displayed when no video is being received.",
    },
    VcpFeatureInfo {
        code: 0xC6,
        name: "Application Enable Key",
        description: "A 2-byte value used to allow an application to only operate with known products. The display manufacturer and application author agree to a code such that application will only run when a valid code is present in the display.",
    },
    VcpFeatureInfo {
        code: 0xC7,
        name: "Display Enable Key",
        description: "This VCP code has been deprecated. It must NOT be implemented in new designs!",
    },
    VcpFeatureInfo {
        code: 0xC8,
        name: "Display Controller ID",
        description: "Contains the ID for the display controller. 1st byte is parsed as the OEM ID, next 3 bytes is a unique chip ID assigned by the OEM.",
    },
    VcpFeatureInfo {
        code: 0xC9,
        name: "Display Firmware Level",
        description: "Contains the firmware version of the display. 1st byte is parsed as the revision number. 2nd byte is the major version. 3rd and 4th are unused.",
    },
    VcpFeatureInfo {
        code: 0xCA,
        name: "OSD / Button Control",
        description: "Sets and indicates the current operational state of the display OSD and buttons",
    },
    VcpFeatureInfo {
        code: 0xCC,
        name: "OSD Language",
        description: "Allows the host to select the display OSD language.",
    },
    VcpFeatureInfo {
        code: 0xCD,
        name: "Status Indicators (Host)",
        description: "This command provides the capability to control up to 16 LED (or similar) indicators which may be used to indicate aspects of the host system status",
    },
    VcpFeatureInfo {
        code: 0xCE,
        name: "Auxiliary Display Size",
        description: "An auxiliary display is a small alphanumeric display associated with the primary display and able to be accessed via the primary display",
    },
    VcpFeatureInfo {
        code: 0xCF,
        name: "Auxiliary Display Data",
        description: "An auxiliary display is a small alphanumeric display associated with the primary display and able to be accessed via the primary display.",
    },
    VcpFeatureInfo {
        code: 0xD0,
        name: "Output Select",
        description: "A one byte write/read (Byte 0), allows the host to set (write) one and only one source to output and identify (read) the current output setting",
    },
    VcpFeatureInfo {
        code: 0xD2,
        name: "Asset Tag",
        description: "This VCP codes allows an Asset Tag to be written to a display or read from a display. It also allows for control by the display manufacturer of which applications may write an asset tag.",
    },
    VcpFeatureInfo {
        code: 0xD4,
        name: "Stereo Video Mode",
        description: "Used to select the video mode with respect to 2D or 3D video.",
    },
    VcpFeatureInfo {
        code: codes::POWER_MODE,
        name: "Power Mode",
        description: "Controls the power mode of the display. 0 = Reserved, 1 = On, 2 = Standby, 3 = Suspend, 4 and 5 = Off",
    },
    VcpFeatureInfo {
        code: 0xD7,
        name: "Auxiliary Power Output",
        description: "Controls output of an auxiliary power output from a display to a host device.",
    },
    VcpFeatureInfo {
        code: 0xDA,
        name: "Scan Mode",
        description: "Controls the scan characteristics.",
    },
    VcpFeatureInfo {
        code: 0xDB,
        name: "Image Mode",
        description: "Controls aspects of the displayed image",
    },
    VcpFeatureInfo {
        code: 0xDC,
        name: "Display Application",
        description: "Select an image preset like Standard, Movie, Games, etc.",
    },
    VcpFeatureInfo {
        code: 0xDE,
        name: "Scratch Pad",
        description: "Provides 2 bytes of volatile storage for use of software application(s) leading to more efficient operation.",
    },
    VcpFeatureInfo {
        code: 0xDF,
        name: "VCP Version",
        description: "Defines the version number of the MCCS standard recognized by the display.",
    },
    // OEM-specific codes (0xE0-0xFF range) - Manufacturer-specific implementations
    VcpFeatureInfo {
        code: 0xE0,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE1,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE2,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE3,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE4,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE5,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE6,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE7,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE8,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xE9,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xEA,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xEB,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xEC,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xED,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xEE,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xEF,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF0,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF1,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF2,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF3,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF4,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF5,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF6,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF7,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF8,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xF9,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFA,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFB,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFC,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFD,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFE,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
    VcpFeatureInfo {
        code: 0xFF,
        name: "OEM specific",
        description: "Manufacturer-specific VCP code",
    },
];

pub fn get_vcp_code_info(code: u8) -> Option<&'static VcpFeatureInfo> {
    KNOWN_VCP_CODES.iter().find(|info| info.code == code)
}
