use bevy::prelude::*;
use reqwest;
use crate::utils::*;
use crate::components::*;

/// System to fetch TLE data from Celestrak
pub async fn fetch_tle_data_system() -> Result<Vec<TleRecord>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Fetch active satellites from Celestrak
    let response = client
        .get("https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle")
        .send()
        .await?;
    
    let tle_text = response.text().await?;
    let records = parse_tle_data(&tle_text)?;
    
    println!("Fetched {} TLE records", records.len());
    Ok(records)
}

/// Resource to hold fetched TLE data
#[derive(Resource)]
pub struct TleDataCache {
    pub records: Vec<TleRecord>,
    pub last_updated: f64,
}

impl Default for TleDataCache {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            last_updated: 0.0,
        }
    }
}

/// System to initialize TLE data on startup
pub fn initialize_tle_data_system(
    mut commands: Commands,
    tle_cache: Res<TleDataCache>,
) {
    // This would normally fetch data, but for now we'll create some test satellites
    if tle_cache.records.is_empty() {
        // Create a few test satellites with realistic orbital parameters
        let test_satellites = vec![
            create_test_satellite("ISS", 25544, 408.0, 51.6),
            create_test_satellite("HUBBLE", 20580, 547.0, 28.5),
            create_test_satellite("GPS BIIR-2", 24876, 20200.0, 55.0),
        ];

        for satellite_data in test_satellites {
            spawn_satellite_entity(&mut commands, satellite_data);
        }
    }
}

fn create_test_satellite(name: &str, norad_id: u32, altitude_km: f64, inclination: f64) -> (TleRecord, OrbitalState) {
    // Create dummy TLE record
    let tle_record = TleRecord {
        name: name.to_string(),
        norad_id,
        classification: 'U',
        international_designator: "00000A".to_string(),
        epoch_year: 23,
        epoch_day: 1.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5, // Rough approximation
        revolution_number: 1,
        line1: "".to_string(),
        line2: "".to_string(),
    };

    // Create initial orbital state
    let orbital_radius = 6371.0 + altitude_km; // Earth radius + altitude
    let orbital_velocity = (3.986004418e14 / (orbital_radius * 1000.0)).sqrt() / 1000.0; // km/s

    let orbital_state = OrbitalState::new(
        Vec3::new(orbital_radius as f32, 0.0, 0.0),
        Vec3::new(0.0, orbital_velocity as f32, 0.0),
        1000.0, // 1000 kg mass
    );

    (tle_record, orbital_state)
}

fn spawn_satellite_entity(
    commands: &mut Commands,
    (tle_record, orbital_state): (TleRecord, OrbitalState),
) {
    commands.spawn((
        Satellite::new(tle_record.name.clone(), tle_record.norad_id, true),
        orbital_state,
        TleData::new(
            tle_record.norad_id,
            tle_record.name.clone(),
            tle_record.line1.clone(),
            tle_record.line2.clone(),
            tle_record.epoch_day,
        ),
        PhysicsObject::satellite(1000.0),
        RenderAsSatellite,
    ));
}