mod autoware_type;
mod vehicle_bridge;

use carla::{
    client::{ActorKind, Client},
    prelude::*,
};
pub use clap::Parser;
use log::info;
use zenoh::prelude::sync::*;

/// Command line options
#[derive(Debug, Clone, Parser)]
struct Opts {
    #[clap(long, default_value = "127.0.0.1")]
    pub carla_address: String,
    #[clap(long, default_value = "2000")]
    pub carla_port: u16,
}

fn main() {
    pretty_env_logger::init();

    let Opts {
        carla_address,
        carla_port,
    } = Opts::parse();

    info!("Running Carla Autoware Zenoh bridge...");
    let z_session = zenoh::open(Config::default()).res().unwrap();

    let client = Client::connect(&carla_address, carla_port, None);
    let world = client.world();

    let mut vehicle_bridge_list = Vec::new();
    for actor in world.actors().iter() {
        match actor.into_kinds() {
            ActorKind::Vehicle(actor) => {
                let role_name = actor
                    .attributes()
                    .iter()
                    .find(|attr| attr.id() == "role_name")
                    .unwrap()
                    .value_string();
                info!("Detect vehicles {}", role_name);
                let v_bridge = vehicle_bridge::VehicleBridge::new(&z_session, role_name, actor);
                vehicle_bridge_list.push(v_bridge);
            }
            ActorKind::Sensor(_) => {
                info!("Detect sensors");
            }
            ActorKind::TrafficLight(_) => {
                info!("Detect traffic light");
            }
            ActorKind::TrafficSign(_) => {
                info!("Detect traffic sign");
            }
            ActorKind::Other(_) => {
                info!("Detect others");
            }
        }
    }

    loop {
        world.wait_for_tick();

        vehicle_bridge_list
            .iter_mut()
            .for_each(|bridge| bridge.step());
    }
}
