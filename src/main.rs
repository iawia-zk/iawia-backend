mod args;
mod db;
mod generator;
mod server;
mod store;
mod types;
mod utils;

use std::collections::HashMap;
use std::path;
use std::sync::Arc;

use aws_nitro_enclaves_nsm_api::driver::{nsm_exit, nsm_init};
use clap::Parser;
use db::{set_witness_generated, update_proof};
use generator::{proof_generator::ProofGenerator, witness_generator::WitnessGenerator};
use jsonrpsee::server::Server;
use server::RpcServer;
use sqlx::postgres::PgPoolOptions;
use utils::{cleanup, get_tmp_folder_path};

#[tokio::main]
async fn main() {
    let config = args::Config::parse();
    let server_url = config.server_address;

    let server = Server::builder().build(server_url).await.unwrap();

    let (file_generator_sender, mut file_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (witness_generator_sender, mut witness_generator_receiver) = tokio::sync::mpsc::channel(10);
    let (proof_generator_sender, mut proof_generator_receiver) = tokio::sync::mpsc::channel(10);

    let fd = 0;
    println!("Running in LOCAL mode (fd = {})", fd);

    println!("Server running on: http://{}", server.local_addr().unwrap());

    let pool = match PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    };

    let circuit_folder = config.circuit_folder.clone();
    let zkey_folder = config.circuit_folder;

    let mut circuit_zkey_map = HashMap::new();

    let circuit_entries = std::fs::read_dir(std::path::Path::new(&circuit_folder)).unwrap();

    for circuit_entry in circuit_entries {
        let circuit_entry = circuit_entry.unwrap();
        let circuit_type_path = circuit_entry.path();
    
        if !circuit_type_path.is_dir() {
            continue;
        }

        let circuit_name = circuit_type_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let zkey_path = path::Path::new(&zkey_folder)
            .join(circuit_type_path.file_name().unwrap())
            .join(format!("{}.zkey", circuit_name));
        let zkey_path_str = zkey_path.to_str().unwrap();

        if !zkey_path.exists() {
            panic!("zkey {zkey_path_str} does not exist!");
        }

        circuit_zkey_map.insert(circuit_name.to_string(), zkey_path_str.to_string());
    }

    let circuit_zkey_map_arc = Arc::new(circuit_zkey_map);

    let handle = server.start(
        server::RpcServerImpl::new(
            fd,
            store::LruStore::new(1000),
            file_generator_sender,
            Arc::clone(&circuit_zkey_map_arc),
            pool.clone(),
        )
        .into_rpc(),
    );

    let rapid_snark_path_exe = path::Path::new(&config.rapidsnark_path)
        .join("build_prover")
        .join("src")
        .join("prover");

    if !rapid_snark_path_exe.exists() {
        panic!("rapidsnark path does not exist!");
    }
    let rapid_snark_path = rapid_snark_path_exe.into_os_string().into_string().unwrap();

    tokio::select! {
        _ = handle.stopped() => {
            println!("Server stopped");
            // nsm_exit(fd);
            println!("Exiting local server");
        }

    _ = async {
        while let Some(file_generator) = file_generator_receiver.recv().await {
            let uuid = file_generator.uuid();

            let pool_clone = pool.clone();
            let witness_generator_clone = witness_generator_sender.clone();
            tokio::spawn(async move {
                let (uuid, circuit_name) = match file_generator.run().await {
                    Ok((uuid, circuit_name)) => (uuid, circuit_name),
                    Err(e) => {
                        dbg!(&e);
                        cleanup(uuid.clone(), &pool_clone, e.to_string()).await;
                        return;
                    }
                };
                if let Err(e) = witness_generator_clone.send(WitnessGenerator::new(
                    uuid.clone(),
                    circuit_name
                )).await {
                    dbg!(&e);
                    cleanup(uuid, &pool_clone, e.to_string()).await;
                    return;
                }
            });
        }
    } => {}

    _ = async {
        while let Some(witness_generator) = witness_generator_receiver.recv().await {
            let circuit_zkey_map_arc_clone = Arc::clone(&circuit_zkey_map_arc);
            let proof_generator_sender_clone = proof_generator_sender.clone();

            let circuit_folder = circuit_folder.clone();
            let zkey_folder = zkey_folder.clone();

            let uuid = witness_generator.uuid.clone();

            let pool_clone = pool.clone();
            tokio::spawn(async move {
                match witness_generator
                    .run(&circuit_folder)
                    .await {
                    Ok((uuid, circuit_name)) => {
                        let zkey_file = circuit_zkey_map_arc_clone.get(circuit_name.as_str()).unwrap();
                        let zkey_file_path = path::Path::new(&zkey_file).to_str().unwrap().to_string();

                        if let Err(e) = set_witness_generated(uuid.clone(), &pool_clone).await {
                            dbg!(&e);
                            cleanup(uuid.clone(), &pool_clone, e.to_string()).await;
                            return;
                        }

                        if let Err(e) = proof_generator_sender_clone.send(ProofGenerator::new(
                            uuid.clone(),
                            zkey_file_path,
                        )).await {
                            dbg!(&e);
                            cleanup(uuid.clone(), &pool_clone, e.to_string()).await;
                            return;
                        }
                    },
                    Err(e) => {
                        dbg!(&e);
                        cleanup(uuid.clone(), &pool_clone, e.to_string()).await;
                        return;
                    }
                }
            });
        }
    } => {}

    _ = async {
        while let Some(proof_generator) = proof_generator_receiver.recv().await {
            let uuid = proof_generator.uuid();

            if let Err(e) = proof_generator.run(&rapid_snark_path).await {
                dbg!(&e);
                cleanup(uuid.clone(), &pool, e.to_string()).await;
                continue;
            }
            if let Err(e) = update_proof(uuid.clone(), &pool).await {
                dbg!(&e);
                cleanup(uuid.clone(), &pool, e.to_string()).await;
                continue;
            }
            let tmp_folder = get_tmp_folder_path(&uuid.to_string());
            let _ = tokio::fs::remove_dir_all(tmp_folder).await;
        }
    } => {}
    }
}
