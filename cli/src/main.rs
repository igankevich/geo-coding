use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use clap::Subcommand;
use geo_coding::Tree2D;
use geo_coding::earth_distance;
use human_units::si::si_unit;
use memmap2::Mmap;

mod pbf;

use self::pbf::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Convert PBF file to RGC file.
    Convert {
        /// Zstd compression level.
        #[clap(long = "compression-level", default_value_t = 9)]
        compression_level: i32,
        file: PathBuf,
    },
    /// Print file contents.
    Show { file: PathBuf },
    /// Find nearest nodes.
    Find {
        #[clap(short = 'f', long = "file")]
        file: PathBuf,
        /// Within which radius to search?
        #[clap(short = 'r', long = "radius", default_value = "10 km")]
        radius: Distance,
        /// How many nodes to return?
        #[clap(short = 'l', long = "limit", default_value_t = 10)]
        limit: usize,
        #[clap(allow_hyphen_values = true)]
        longitude: f64,
        #[clap(allow_hyphen_values = true)]
        latitude: f64,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[si_unit(symbol = "m", min_prefix = "", max_prefix = "k")]
pub struct Distance(pub u64);

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Convert {
            compression_level,
            file,
        } => {
            let t = Instant::now();
            let ImportedNodes {
                other,
                settlements,
                countries,
            } = ImportedNodes::import_pbf(&file)?;
            eprintln!("Import PBF: {:?}", t.elapsed());
            let t = Instant::now();
            for (tree, filename) in [
                (other, "other.rgc.zst"),
                (settlements, "settlements.rgc.zst"),
                (countries, "countries.rgc.zst"),
            ] {
                let file = fs::File::create(filename)?;
                // TODO zstd cli compresses two times better...
                let mut encoder = zstd::Encoder::new(file, compression_level)?;
                tree.write(&mut encoder)?;
                encoder.finish()?;
            }
            eprintln!("Encoding: {:?}", t.elapsed());
        }
        Command::Show { file } => {
            let file = fs::File::open(&file)?;
            let file = zstd::Decoder::new(file)?;
            let tree = Tree2D::<i64, String>::read(file)?;
            for ([longitude, latitude], name) in tree.iter() {
                println!(
                    "{:.9} {:.9} {name}",
                    *longitude as f64 * 1e-9,
                    *latitude as f64 * 1e-9
                );
            }
        }
        Command::Find {
            file,
            longitude,
            latitude,
            radius,
            limit,
        } => {
            let t = Instant::now();
            let file = fs::File::open(&file)?;
            let mmap = unsafe { Mmap::map(&file)?  };
            let file = zstd::Decoder::new(mmap.as_ref())?;
            let geocoder = Tree2D::<i64, String>::read(file)?;
            eprintln!("Open: {:?}", t.elapsed());
            let t = Instant::now();
            let location = [(longitude * 1e9) as i64, (latitude * 1e9) as i64];
            let neighbours = geocoder.find_nearest(&location, radius.0, limit, earth_distance);
            eprintln!("Search: {:?}", t.elapsed());
            for (_distance, [longitude, latitude], name) in neighbours.iter() {
                println!(
                    "{:.9} {:.9} {name}",
                    *longitude as f64 * 1e-9,
                    *latitude as f64 * 1e-9
                );
            }
        }
    }
    Ok(())
}
