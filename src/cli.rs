use crate::{Result, monitor, monitor::Monitor, vcp};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "monitor-config")]
#[command(author, version, about = "Native Windows CLI tool for managing monitor settings", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all available monitors
    List {
        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Get brightness level of a monitor
    GetBrightness {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Set brightness level of a monitor
    SetBrightness {
        /// Brightness value (0-100)
        value: u32,

        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,
    },

    /// Get contrast level of a monitor
    GetContrast {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Set contrast level of a monitor
    SetContrast {
        /// Contrast value (0-100)
        value: u32,

        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,
    },

    /// Get VCP feature value
    GetVcp {
        /// VCP code (e.g., 0x10 for brightness)
        #[arg(value_parser = parse_hex)]
        code: u8,

        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Set VCP feature value
    SetVcp {
        /// VCP code (e.g., 0x10 for brightness)
        #[arg(value_parser = parse_hex)]
        code: u8,

        /// Value to set
        value: u32,

        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,
    },

    /// List all VCP codes
    ListVcp {
        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Scan monitor for all supported VCP codes
    ScanVcp {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Get monitor capabilities string
    GetCapabilities {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,
    },

    /// Save current monitor settings
    SaveSettings {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,
    },

    /// Reset monitor to factory defaults
    ResetDefaults {
        /// Device name (e.g., \\.\DISPLAY1) or use --primary
        #[arg(short, long)]
        device: Option<String>,

        /// Use primary monitor
        #[arg(short, long)]
        primary: bool,

        /// Only reset color settings
        #[arg(short, long)]
        color_only: bool,
    },
}

fn parse_hex(s: &str) -> std::result::Result<u8, String> {
    if let Some(stripped) = s.strip_prefix("0x") {
        u8::from_str_radix(stripped, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u8>().map_err(|e| e.to_string())
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { json } => list_monitors(json),
        Commands::GetBrightness {
            device,
            primary,
            json,
        } => get_brightness(device, primary, json),
        Commands::SetBrightness {
            value,
            device,
            primary,
        } => set_brightness(value, device, primary),
        Commands::GetContrast {
            device,
            primary,
            json,
        } => get_contrast(device, primary, json),
        Commands::SetContrast {
            value,
            device,
            primary,
        } => set_contrast(value, device, primary),
        Commands::GetVcp {
            code,
            device,
            primary,
            json,
        } => get_vcp(code, device, primary, json),
        Commands::SetVcp {
            code,
            value,
            device,
            primary,
        } => set_vcp(code, value, device, primary),
        Commands::ListVcp { json } => list_vcp(json),
        Commands::ScanVcp {
            device,
            primary,
            json,
        } => scan_vcp(device, primary, json),
        Commands::GetCapabilities { device, primary } => get_capabilities(device, primary),
        Commands::SaveSettings { device, primary } => save_settings(device, primary),
        Commands::ResetDefaults {
            device,
            primary,
            color_only,
        } => reset_defaults(device, primary, color_only),
    }
}

fn get_monitor(device: Option<String>, primary: bool) -> Result<monitor::PhysicalMonitor> {
    if primary {
        monitor::get_primary_monitor()
    } else if let Some(device_name) = device {
        monitor::find_monitor(&device_name)
    } else {
        monitor::get_primary_monitor()
    }
}

fn list_monitors(json: bool) -> Result<()> {
    let monitors = monitor::enumerate_monitors()?;

    if json {
        let info: Vec<_> = monitors.iter().map(|m| m.info()).collect();
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!(
            "{:<20} {:<30} {}",
            "Device Name", "Friendly Name", "Primary"
        );
        println!("{}", "-".repeat(70));
        for mon in &monitors {
            let info = mon.info();
            println!(
                "{:<20} {:<30} {}",
                info.device_name,
                info.friendly_name,
                if info.is_primary { "Yes" } else { "" }
            );
        }
    }

    Ok(())
}

fn get_brightness(device: Option<String>, primary: bool, json: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let brightness = mon.get_brightness()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&brightness)?);
    } else {
        println!(
            "Current brightness: {} (min: {}, max: {})",
            brightness.current, brightness.minimum, brightness.maximum
        );
    }

    Ok(())
}

fn set_brightness(value: u32, device: Option<String>, primary: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    mon.set_brightness(value)?;
    println!("Brightness set to {}", value);
    Ok(())
}

fn get_contrast(device: Option<String>, primary: bool, json: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let contrast = mon.get_contrast()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&contrast)?);
    } else {
        println!(
            "Current contrast: {} (min: {}, max: {})",
            contrast.current, contrast.minimum, contrast.maximum
        );
    }

    Ok(())
}

fn set_contrast(value: u32, device: Option<String>, primary: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    mon.set_contrast(value)?;
    println!("Contrast set to {}", value);
    Ok(())
}

fn get_vcp(code: u8, device: Option<String>, primary: bool, json: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());
    let response = vcp_mon.get_vcp_feature(code)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        let info = vcp::get_vcp_code_info(code);
        if let Some(info) = info {
            println!("VCP Code: 0x{:02X} - {}", code, info.name);
            println!("Description: {}", info.description);
        } else {
            println!("VCP Code: 0x{:02X}", code);
        }
        println!(
            "Current value: {} (max: {})",
            response.current_value, response.maximum_value
        );
        println!("Type: {:?}", response.code_type);
    }

    Ok(())
}

fn set_vcp(code: u8, value: u32, device: Option<String>, primary: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());
    vcp_mon.set_vcp_feature(code, value)?;
    println!("VCP code 0x{:02X} set to {}", code, value);
    Ok(())
}

fn list_vcp(json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&vcp::KNOWN_VCP_CODES)?);
    } else {
        println!("{:<6} {:<30} {}", "Code", "Name", "Description");
        println!("{}", "-".repeat(80));
        for info in vcp::KNOWN_VCP_CODES {
            println!(
                "0x{:02X}   {:<30} {}",
                info.code, info.name, info.description
            );
        }
    }
    Ok(())
}

fn scan_vcp(device: Option<String>, primary: bool, json: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());
    
    if !json {
        eprintln!("Scanning monitor for supported VCP codes...");
    }
    
    let features = vcp_mon.scan_vcp_features();
    
    if json {
        println!("{}", serde_json::to_string_pretty(&features)?);
    } else {
        eprintln!("Found {} supported VCP codes\n", features.len());
        println!("{:<6} {:<35} {:<12} {:<8} {}", "Code", "Name", "CurrentValue", "MaxValue", "Description");
        println!("{}", "-".repeat(120));
        
        for response in features {
            let info = vcp::get_vcp_code_info(response.vcp_code);
            let name = info.map(|i| i.name).unwrap_or("Unknown");
            let description = info.map(|i| i.description).unwrap_or("");
            
            println!(
                "0x{:02X}   {:<35} {:<12} {:<8} {}",
                response.vcp_code,
                name,
                response.current_value,
                response.maximum_value,
                description
            );
        }
    }
    
    Ok(())
}

fn get_capabilities(device: Option<String>, primary: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());
    let caps = vcp_mon.get_capabilities()?;
    println!("{}", caps);
    Ok(())
}

fn save_settings(device: Option<String>, primary: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());
    vcp_mon.save_settings()?;
    println!("Monitor settings saved");
    Ok(())
}

fn reset_defaults(device: Option<String>, primary: bool, color_only: bool) -> Result<()> {
    let mon = get_monitor(device, primary)?;
    let vcp_mon = vcp::VcpMonitor::new(mon.handle());

    if color_only {
        vcp_mon.restore_factory_color_defaults()?;
        println!("Monitor color settings reset to factory defaults");
    } else {
        vcp_mon.restore_factory_defaults()?;
        println!("Monitor reset to factory defaults");
    }

    Ok(())
}
