use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TleRecord {
    pub name: String,
    pub norad_id: u32,
    pub classification: char,
    pub international_designator: String,
    pub epoch_year: u32,
    pub epoch_day: f64,
    pub mean_motion_dot: f64,
    pub mean_motion_ddot: f64,
    pub bstar: f64,
    pub inclination: f64,
    pub right_ascension: f64,
    pub eccentricity: f64,
    pub argument_of_perigee: f64,
    pub mean_anomaly: f64,
    pub mean_motion: f64,
    pub revolution_number: u32,
    pub line1: String,
    pub line2: String,
}

impl TleRecord {
    /// Parse a complete TLE record from three lines
    pub fn from_tle_lines(name: &str, line1: &str, line2: &str) -> Result<Self, TleParseError> {
        if line1.len() < 69 || line2.len() < 69 {
            return Err(TleParseError::InvalidLength);
        }

        // Parse Line 1
        let norad_id = line1[2..7].trim().parse::<u32>()
            .map_err(|_| TleParseError::InvalidField("NORAD ID".to_string()))?;
        
        let classification = line1.chars().nth(7).unwrap_or('U');
        let international_designator = line1[9..17].trim().to_string();
        
        let epoch_year = line1[18..20].trim().parse::<u32>()
            .map_err(|_| TleParseError::InvalidField("Epoch Year".to_string()))?;
        
        let epoch_day = line1[20..32].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Epoch Day".to_string()))?;

        let mean_motion_dot = parse_signed_decimal(&line1[33..43])?;
        let mean_motion_ddot = parse_exponential(&line1[44..52])?;
        let bstar = parse_exponential(&line1[53..61])?;

        // Parse Line 2
        let inclination = line2[8..16].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Inclination".to_string()))?;
        
        let right_ascension = line2[17..25].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Right Ascension".to_string()))?;
        
        let eccentricity = parse_decimal_fraction(&line2[26..33])?;
        
        let argument_of_perigee = line2[34..42].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Argument of Perigee".to_string()))?;
        
        let mean_anomaly = line2[43..51].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Mean Anomaly".to_string()))?;
        
        let mean_motion = line2[52..63].trim().parse::<f64>()
            .map_err(|_| TleParseError::InvalidField("Mean Motion".to_string()))?;
        
        let revolution_number = line2[63..68].trim().parse::<u32>()
            .map_err(|_| TleParseError::InvalidField("Revolution Number".to_string()))?;

        Ok(TleRecord {
            name: name.trim().to_string(),
            norad_id,
            classification,
            international_designator,
            epoch_year,
            epoch_day,
            mean_motion_dot,
            mean_motion_ddot,
            bstar,
            inclination,
            right_ascension,
            eccentricity,
            argument_of_perigee,
            mean_anomaly,
            mean_motion,
            revolution_number,
            line1: line1.to_string(),
            line2: line2.to_string(),
        })
    }
}

/// Parse TLE data from a multi-line string
pub fn parse_tle_data(data: &str) -> Result<Vec<TleRecord>, TleParseError> {
    let lines: Vec<&str> = data.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let mut records = Vec::new();
    
    let mut i = 0;
    while i + 2 < lines.len() {
        // TLE format: name line, line 1, line 2
        if lines[i + 1].starts_with('1') && lines[i + 2].starts_with('2') {
            match TleRecord::from_tle_lines(lines[i], lines[i + 1], lines[i + 2]) {
                Ok(record) => records.push(record),
                Err(e) => {
                    debug!("Failed to parse TLE record for {}: {:?}", lines[i], e);
                }
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    
    Ok(records)
}

// Helper parsing functions
fn parse_signed_decimal(s: &str) -> Result<f64, TleParseError> {
    s.trim().parse::<f64>()
        .map_err(|_| TleParseError::InvalidField("Signed decimal".to_string()))
}

fn parse_exponential(s: &str) -> Result<f64, TleParseError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "00000-0" {
        return Ok(0.0);
    }
    
    // Handle TLE exponential notation like "12345-3" -> "0.12345e-3"
    if let Some(exp_pos) = trimmed.rfind(['+', '-']) {
        if exp_pos > 0 {
            let mantissa_str = &trimmed[..exp_pos];
            let exponent_str = &trimmed[exp_pos..];
            
            let mantissa: f64 = mantissa_str.parse()
                .map_err(|_| TleParseError::InvalidField("Exponential mantissa".to_string()))?;
            let exponent: i32 = exponent_str.parse()
                .map_err(|_| TleParseError::InvalidField("Exponential exponent".to_string()))?;
            
            return Ok(mantissa * 10f64.powi(exponent - mantissa_str.len() as i32 + 1));
        }
    }
    
    trimmed.parse::<f64>()
        .map_err(|_| TleParseError::InvalidField("Exponential".to_string()))
}

fn parse_decimal_fraction(s: &str) -> Result<f64, TleParseError> {
    let trimmed = s.trim();
    let value: f64 = trimmed.parse()
        .map_err(|_| TleParseError::InvalidField("Decimal fraction".to_string()))?;
    Ok(value / 10f64.powi(trimmed.len() as i32))
}

#[derive(Debug, Clone)]
pub enum TleParseError {
    InvalidLength,
    InvalidField(String),
    InvalidFormat,
}

impl std::fmt::Display for TleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TleParseError::InvalidLength => write!(f, "TLE line has invalid length"),
            TleParseError::InvalidField(field) => write!(f, "Invalid field: {}", field),
            TleParseError::InvalidFormat => write!(f, "Invalid TLE format"),
        }
    }
}

impl std::error::Error for TleParseError {}