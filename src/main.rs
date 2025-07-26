use std::str::FromStr;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rustcolors",
    about = "Color manipulation utilities for scientific visualization",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display color harmonies (complement, triads, tetrads)
    Harmonies {
        /// Input color in hex (#RRGGBB/RRGGBB) or RGB (r,g,b) format
        #[arg(help = "Input color in hex (#RRGGBB/RRGGBB) or RGB (r,g,b) format")]
        color: String,
    },
    /// Convert between color formats
    Convert {
        /// Input color in hex (#RRGGBB/RRGGBB) or RGB (r,g,b) format
        #[arg(help = "Input color in hex (#RRGGBB/RRGGBB) or RGB (r,g,b) format")]
        color: String,
        /// Output format (hex, rgb, hsl)
        #[arg(value_parser = ["hex", "rgb", "hsl"])]
        format: String,
    },
}

#[derive(Debug, Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, Copy)]
struct HSL {
    h: f64,
    s: f64,
    l: f64,
}

impl RGB {
    fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    fn to_ansi_color_block(&self) -> String {
        // Create a wider color block using RGB background color
        format!("\x1b[48;2;{};{};{}m        \x1b[0m", self.r, self.g, self.b)
    }

    fn display_with_color(&self) -> String {
        format!("{} {}", self.to_ansi_color_block(), self.to_hex())
    }

    fn to_hsl(&self) -> HSL {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        let mut h = 0.0;
        let mut s = 0.0;
        let l = (max + min) / 2.0;

        if delta != 0.0 {
            s = if l < 0.5 {
                delta / (max + min)
            } else {
                delta / (2.0 - max - min)
            };

            h = match max {
                x if x == r => ((g - b) / delta) + (if g < b { 6.0 } else { 0.0 }),
                x if x == g => ((b - r) / delta) + 2.0,
                _ => ((r - g) / delta) + 4.0,
            };

            h *= 60.0;
        }

        HSL {
            h,
            s: s * 100.0,
            l: l * 100.0,
        }
    }

    fn complement(&self) -> RGB {
        RGB {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }

    fn rotate_hue(&self, degrees: f64) -> RGB {
        let mut hsl = self.to_hsl();
        hsl.h = (hsl.h + degrees) % 360.0;
        hsl.to_rgb()
    }

    fn triads(&self) -> Vec<RGB> {
        vec![
            *self,
            self.rotate_hue(120.0),
            self.rotate_hue(240.0),
        ]
    }

    fn tetrads(&self) -> Vec<RGB> {
        vec![
            *self,
            self.rotate_hue(90.0),
            self.rotate_hue(180.0),
            self.rotate_hue(270.0),
        ]
    }
}

impl HSL {
    fn to_rgb(&self) -> RGB {
        let h = self.h / 360.0;
        let s = self.s / 100.0;
        let l = self.l / 100.0;

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> u8 {
            if t < 0.0 { t += 1.0 }
            if t > 1.0 { t -= 1.0 }

            let color = if t < 1.0/6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0/2.0 {
                q
            } else if t < 2.0/3.0 {
                p + (q - p) * (2.0/3.0 - t) * 6.0
            } else {
                p
            };

            (color * 255.0).round() as u8
        }

        RGB {
            r: hue_to_rgb(p, q, h + 1.0/3.0),
            g: hue_to_rgb(p, q, h),
            b: hue_to_rgb(p, q, h - 1.0/3.0),
        }
    }
}

impl FromStr for RGB {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        
        // Handle hex format (with or without #)
        if s.starts_with('#') || s.chars().all(|c| c.is_ascii_hexdigit()) {
            let hex = if s.starts_with('#') { &s[1..] } else { s };
            
            if hex.len() != 6 {
                return Err("Invalid hex color format. Expected RRGGBB or #RRGGBB".to_string());
            }
            
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| "Invalid red component")?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| "Invalid green component")?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| "Invalid blue component")?;

            Ok(RGB { r, g, b })
        } else {
            // Parse RGB format (r,g,b)
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() != 3 {
                return Err("Invalid RGB format. Expected r,g,b".to_string());
            }

            let r = parts[0].trim().parse()
                .map_err(|_| "Invalid red component")?;
            let g = parts[1].trim().parse()
                .map_err(|_| "Invalid green component")?;
            let b = parts[2].trim().parse()
                .map_err(|_| "Invalid blue component")?;

            Ok(RGB { r, g, b })
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Harmonies { color } => {
            let rgb = RGB::from_str(&color).unwrap_or_else(|e| {
                eprintln!("Error parsing color: {}", e);
                std::process::exit(1);
            });

            println!("\nColor Harmonies for Input: {}", rgb.display_with_color());
            println!("Complement: {}", rgb.complement().display_with_color());
            
            println!("\nTriads:");
            for color in rgb.triads() {
                println!("  {}", color.display_with_color());
            }

            println!("\nTetrads:");
            for color in rgb.tetrads() {
                println!("  {}", color.display_with_color());
            }
        },
        Commands::Convert { color, format } => {
            let rgb = RGB::from_str(&color).unwrap_or_else(|e| {
                eprintln!("Error parsing color: {}", e);
                std::process::exit(1);
            });

            match format.to_lowercase().as_str() {
                "hex" => println!("{}", rgb.display_with_color()),
                "rgb" => println!("{} RGB({}, {}, {})", 
                    rgb.to_ansi_color_block(), rgb.r, rgb.g, rgb.b),
                "hsl" => {
                    let hsl = rgb.to_hsl();
                    println!("{} HSL({:.1}, {:.1}%, {:.1}%)", 
                        rgb.to_ansi_color_block(), hsl.h, hsl.s, hsl.l);
                },
                _ => unreachable!(), // clap validates the format for us
            }
        },
    }
}
