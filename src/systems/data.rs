use bevy::prelude::*;
use reqwest;
use crate::utils::*;
use crate::components::*;
use crate::utils::sgp4_wrapper::*;

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

/// System to initialize TLE data on startup by fetching from Celestrak
pub fn initialize_tle_data_system(
    mut commands: Commands,
    mut tle_cache: ResMut<TleDataCache>,
) {
    // Only fetch if we don't have data yet
    if tle_cache.records.is_empty() {
        println!("Initializing satellite data...");
        commands.spawn_empty().insert(TleFetchTask);
    }
}

/// Marker component for TLE fetch task
#[derive(Component)]
pub struct TleFetchTask;

/// System to handle TLE data fetching from Celestrak
pub fn process_tle_fetch_system(
    mut commands: Commands,
    mut tle_cache: ResMut<TleDataCache>,
    query: Query<Entity, With<TleFetchTask>>,
) {
    for entity in query.iter() {
        // Remove the fetch task marker
        commands.entity(entity).despawn();
        
        println!("Attempting to fetch live TLE data from Celestrak...");
        
        // Try to fetch live data, fallback to expanded test data if it fails
        match try_fetch_live_tle_data() {
            Ok(records) => {
                // Take only the first 20 satellites for Phase 2
                let limited_records: Vec<_> = records.into_iter().take(20).collect();
                println!("Successfully fetched {} TLE records from Celestrak", limited_records.len());
                
                // Store in cache
                tle_cache.records = limited_records.clone();
                tle_cache.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                
                // Spawn satellites from live TLE data
                spawn_satellites_from_records(&mut commands, &limited_records);
            }
            Err(e) => {
                eprintln!("Failed to fetch live TLE data: {}", e);
                println!("Using extended test satellite dataset...");
                
                // Use expanded test dataset with 20 realistic satellites
                create_extended_test_dataset(&mut commands, &mut tle_cache);
            }
        }
    }
}

/// Try to fetch live TLE data from Celestrak (blocking call)
fn try_fetch_live_tle_data() -> Result<Vec<TleRecord>, String> {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn a thread for the async operation
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            match fetch_tle_data_system().await {
                Ok(records) => Ok(records),
                Err(e) => Err(e.to_string()),
            }
        });
        let _ = tx.send(result);
    });
    
    // Wait for result with timeout
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(records)) => Ok(records),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout fetching TLE data".to_string()),
    }
}

/// Create extended test dataset with 20 realistic satellites
fn create_extended_test_dataset(commands: &mut Commands, tle_cache: &mut ResMut<TleDataCache>) {
    let test_satellites = vec![
        // Low Earth Orbit satellites
        create_test_satellite("ISS (ZARYA)", 25544, 408.0, 51.6),
        create_test_satellite("HUBBLE SPACE TELESCOPE", 20580, 547.0, 28.5),
        create_test_satellite("TERRA", 25994, 705.0, 98.2),
        create_test_satellite("AQUA", 27424, 705.0, 98.2),
        create_test_satellite("NOAA-18", 28654, 854.0, 98.9),
        create_test_satellite("METOP-A", 29499, 817.0, 98.7),
        create_test_satellite("WORLDVIEW-1", 32060, 496.0, 97.2),
        create_test_satellite("KEPLER", 36411, 621.0, 89.0),
        create_test_satellite("SENTINEL-1A", 39634, 693.0, 98.2),
        create_test_satellite("LANDSAT-8", 39084, 705.3, 98.2),
        
        // Medium Earth Orbit satellites
        create_test_satellite("GPS BIIR-2 (PRN 13)", 24876, 20200.0, 55.0),
        create_test_satellite("GPS BIIR-10 (PRN 12)", 32260, 20200.0, 55.0),
        create_test_satellite("GALILEO-FOC FM14", 41549, 23222.0, 56.0),
        create_test_satellite("GLONASS-M 758", 36111, 19130.0, 64.8),
        create_test_satellite("BEIDOU-3 M15", 43581, 21528.0, 55.0),
        
        // High Earth Orbit / GEO satellites
        create_test_satellite("JASON-2", 33105, 1336.0, 66.0),
        create_test_satellite("JASON-3", 41240, 1336.0, 66.0),
        create_test_satellite("GOES-16", 41866, 35786.0, 0.1),
        create_test_satellite("INTELSAT 29E", 41308, 35786.0, 0.1),
        create_test_satellite("ASTRA 2E", 38087, 35786.0, 0.1),
    ];
    
    // Store test TLE records in cache
    for (tle_record, _orbital_state) in &test_satellites {
        tle_cache.records.push(tle_record.clone());
    }
    
    // Spawn satellite entities
    spawn_satellites_from_records(commands, &tle_cache.records);
    
    println!("Created extended test dataset with {} satellites", test_satellites.len());
}

/// Spawn satellites from TLE records
fn spawn_satellites_from_records(commands: &mut Commands, records: &[TleRecord]) {
    let mut spawned_count = 0;
    let mut failed_count = 0;
    
    for tle_record in records {
        match create_satellite_from_tle(tle_record) {
            Ok(satellite_data) => {
                spawn_satellite_entity(commands, satellite_data);
                spawned_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to create satellite {}: {}", tle_record.name, e);
                failed_count += 1;
            }
        }
    }
    
    println!("Spawned {} satellites ({} failed)", spawned_count, failed_count);
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

/// Create satellite from real TLE data using SGP4 conversion
pub fn create_satellite_from_tle(tle_record: &TleRecord) -> Result<(TleRecord, OrbitalState), String> {
    // Use SGP4 to convert TLE to position/velocity state vectors
    let (position, velocity) = tle_to_state_vectors(tle_record)?;
    
    // Estimate mass based on satellite type (this is a simplification)
    // In reality, mass would come from satellite databases
    let estimated_mass = estimate_satellite_mass(&tle_record.name);
    
    let orbital_state = OrbitalState::new(position, velocity, estimated_mass);
    
    Ok((tle_record.clone(), orbital_state))
}

/// Estimate satellite mass based on name/type (simplified heuristic)
fn estimate_satellite_mass(name: &str) -> f64 {
    let name_upper = name.to_uppercase();
    
    // Mass estimates in kg based on satellite types
    if name_upper.contains("ISS") || name_upper.contains("ZARYA") {
        450000.0 // International Space Station
    } else if name_upper.contains("HUBBLE") {
        11110.0  // Hubble Space Telescope
    } else if name_upper.contains("GPS") {
        2030.0   // GPS satellites
    } else if name_upper.contains("TERRA") || name_upper.contains("AQUA") {
        5190.0   // Earth observation satellites
    } else if name_upper.contains("STARLINK") {
        260.0    // Starlink satellites
    } else if name_upper.contains("IRIDIUM") {
        689.0    // Iridium satellites
    } else if name_upper.contains("GLOBALSTAR") {
        450.0    // Globalstar satellites
    } else if name_upper.contains("SPOT") || name_upper.contains("LANDSAT") {
        2200.0   // Earth imaging satellites
    } else {
        1000.0   // Default estimate for unknown satellites
    }
}

/// System to spawn satellites from live TLE data (for future use)
pub fn spawn_satellites_from_tle_data(
    mut commands: Commands,
    tle_cache: Res<TleDataCache>,
    existing_sats: Query<&Satellite>,
) {
    // Check if we already have satellites spawned
    if existing_sats.iter().count() > 0 {
        return; // Already have satellites spawned
    }
    
    println!("Creating satellites from TLE data...");
    let mut spawned_count = 0;
    let mut failed_count = 0;
    
    // Limit to first 100 satellites for Phase 2 testing
    // In production, this could be configurable
    let max_satellites = 100;
    
    for tle_record in tle_cache.records.iter().take(max_satellites) {
        match create_satellite_from_tle(tle_record) {
            Ok(satellite_data) => {
                spawn_satellite_entity(&mut commands, satellite_data);
                spawned_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to create satellite {}: {}", tle_record.name, e);
                failed_count += 1;
            }
        }
    }
    
    println!("Spawned {} satellites from TLE data ({} failed)", spawned_count, failed_count);
}