use clap::{Parser, Subcommand, value_parser};

#[derive(Parser, Debug)]
#[clap(subcommand_required = true, arg_required_else_help = true)]
#[command(version, about = "WireGuard Mesh Configuration File Generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Config file path
    #[arg(short, long)]
    pub config: Box<str>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Init a mesh config file")]
    Init {
        /// Number of mesh objects to initialize
        #[arg(short, long, value_parser = value_parser!(u32).range(0..=16_777_214))]
        count: Option<u32>,
    },

    #[command(about = "Convert mesh config to wireguard config")]
    Convert {
        /// Output directory
        #[arg(short, long)]
        output: Box<str>,
    },

    #[command(about = "Append a `mesh` to the config")]
    Append {
        /// The tag of appending `mesh`
        #[arg(short, long)]
        tag: Box<str>,
        /// Number of appending `meshes`
        #[arg(short, long, value_parser = value_parser!(u32).range(0..=16_777_214))]
        count: Option<u32>,
        /// Turn on edit in-place mode
        #[arg(short, long)]
        in_place: bool,
    },
}
