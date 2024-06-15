use serde::Deserialize;
use utxorpc_spec::utxorpc::v1alpha::sync::chain_sync_service_client::ChainSyncServiceClient;
use utxorpc_spec::utxorpc::v1alpha::sync::BlockRef;
use utxorpc_spec::utxorpc::v1alpha::sync::DumpHistoryRequest;
use utxorpc_spec::utxorpc::v1alpha::sync::FetchBlockRequest;
use utxorpc_spec::utxorpc::v1alpha::sync::FollowTipRequest;

use clap::{Parser, Subcommand};
use config::{Config, Environment, File};
use serde_json::json;
use tonic::Request;

#[derive(Parser)]
#[clap(
    name = "u5client",
    about = "Lightweight CLI tool for interacting with UTXO RPC APIs",
    version
)]
struct Cli {
    #[clap(
        short,
        long,
        default_value = "config.toml",
        help = "Path to the configuration file"
    )]
    config: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Deserialize, Debug)]
struct RootConfig {
    peer: String,
    save_dir: String,
}

#[derive(Subcommand)]
enum Commands {
    Fetch {
        #[clap(long, help = "Block references to fetch", value_parser, num_args = 1.., value_delimiter = ' ')]
        refs: Vec<String>,
        #[clap(long, help = "Save fetched blocks to files")]
        save: bool,
    },
    Dump {
        #[clap(long, help = "Block reference to start dumping from")]
        r#ref: String,
        #[clap(long, help = "Number of blocks to dump")]
        num_blocks: u32,
        #[clap(long, help = "Save dumped blocks to files")]
        save: bool,
    },
    Follow {
        #[clap(long, help = "Block references to try and intersect", value_parser, num_args = 1.., value_delimiter = ' ')]
        refs: Option<Vec<String>>,
    },
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut settings = Config::builder();
    settings = settings.add_source(File::with_name(&cli.config));
    settings = settings.add_source(Environment::with_prefix("U5CLIENT").separator("__"));
    let config = settings.build()?.try_deserialize::<RootConfig>()?;

    println!("Using configuration: {:?}", config);

    match &cli.command {
        Commands::Dump {
            r#ref,
            num_blocks,
            save,
        } => {
            let mut client = ChainSyncServiceClient::connect(config.peer).await?;
            let parts: Vec<&str> = r#ref.split('-').collect();

            let dump_history_request = DumpHistoryRequest {
                start_token: Some(BlockRef {
                    index: parts[0].parse().unwrap(),
                    hash: hex::decode(parts[1]).unwrap().into(),
                }),
                max_items: *num_blocks,
                field_mask: None,
            };

            let response = client
                .dump_history(Request::new(dump_history_request))
                .await?
                .into_inner();

            for any_chain_block in response.block.into_iter() {
                let (index, block_data) = match any_chain_block.chain.unwrap() {
                    utxorpc_spec::utxorpc::v1alpha::sync::any_chain_block::Chain::Cardano(
                        block,
                    ) => (block.header.clone().unwrap().slot, json!(block)),
                    _ => continue,
                };

                if *save {
                    let filename = format!("{}/{}.json", config.save_dir, index);
                    let mut file = std::fs::File::create(filename)?;
                    serde_json::to_writer_pretty(&mut file, &block_data)?;
                } else {
                    println!("Block {}: {:?}", index, json!(&block_data));
                }
            }
        }
        Commands::Fetch { refs, save } => {
            let mut client = ChainSyncServiceClient::connect(config.peer).await?;
            let block_refs: Vec<BlockRef> = refs
                .iter()
                .map(|ref_str| {
                    let parts: Vec<&str> = ref_str.split('-').collect();
                    BlockRef {
                        index: parts[0].parse().unwrap(),
                        hash: hex::decode(parts[1]).unwrap().into(),
                    }
                })
                .collect();

            let request = FetchBlockRequest {
                r#ref: block_refs,
                field_mask: None,
            };

            let response = client
                .fetch_block(Request::new(request))
                .await?
                .into_inner();

            for any_chain_block in response.block.into_iter() {
                let (index, block_data) = match any_chain_block.chain.unwrap() {
                    utxorpc_spec::utxorpc::v1alpha::sync::any_chain_block::Chain::Cardano(
                        block,
                    ) => (block.header.clone().unwrap().slot, json!(block)),
                    _ => continue,
                };

                if *save {
                    let filename = format!("{}/{}.json", config.save_dir, index);
                    let mut file = std::fs::File::create(filename)?;
                    serde_json::to_writer_pretty(&mut file, &block_data)?;
                } else {
                    println!("Block {}: {:?}", index, json!(&block_data));
                }
            }
        }
        Commands::Follow { refs } => {
            let mut client = ChainSyncServiceClient::connect(config.peer).await?;

            let intersect_refs: Vec<BlockRef> = refs
                .as_ref()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|intersect| {
                    let parts: Vec<&str> = intersect.split('-').collect();
                    BlockRef {
                        index: parts[0].parse().unwrap(),
                        hash: hex::decode(parts[1]).unwrap().into(),
                    }
                })
                .collect();

            let request = FollowTipRequest {
                intersect: intersect_refs,
            };

            let mut stream = client.follow_tip(Request::new(request)).await?.into_inner();

            while let Some(response) = stream.message().await? {
                match response.action.unwrap() {
                    utxorpc_spec::utxorpc::v1alpha::sync::follow_tip_response::Action::Apply(
                        block,
                    ) => {
                        println!("Apply this block: {:?}", json!(&block));
                    }
                    utxorpc_spec::utxorpc::v1alpha::sync::follow_tip_response::Action::Undo(
                        block,
                    ) => {
                        println!("Undo this block: {:?}", json!(&block));
                    }
                    utxorpc_spec::utxorpc::v1alpha::sync::follow_tip_response::Action::Reset(
                        ref_block,
                    ) => {
                        println!("Reset to this block reference: {:?}", ref_block);
                    }
                }
            }
        }
    }

    Ok(())
}
