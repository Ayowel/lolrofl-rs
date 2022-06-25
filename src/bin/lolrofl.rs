use clap::{Args, ArgEnum, Parser, Subcommand};
use json::parse;
use lolrofl::{Rofl, model::section::{GenericSection, SectionCore}};

/// A program to extract information from LoL replay files
#[derive(Parser, Debug)]
#[clap(author = "Ayowel", version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: CliCommands,

    /// Path to the ROFL file to open
    #[clap(global=true)]
    file: Option<std::path::PathBuf>,

    /// Verbose mode
    #[clap(short, long, global=true)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum CliCommands {
    #[clap(about = "Get information on the file")]
    Get(InspectCommand),
    #[clap(about = "Get information on the file")]
    Analyze(AnalyzeCommand),
    #[clap(about = "Export chunk or keyframe data to a file")]
    Export(ExportCommand),
}

#[derive(Debug, Args)]
struct InspectCommand {
    #[clap(subcommand)]
    command: SubInspectCommands,
}

#[derive(Debug, Subcommand)]
enum SubInspectCommands {
    #[clap(alias = "i", about = "Print simple/high-level info on the file and the game")]
    Info(InfoInspectCommand),
    #[clap(alias = "m", about = "Print the game's metadata")]
    Metadata(MetadataInspectCommand),
    #[clap(alias = "p", about = "Print technical information on the file")]
    Payload(PayloadInspectCommand),
    #[clap(alias = "r", about = "NOT IMPLEMENTED - Print details on exported payload data")]
    RawData(RawDataInspectCommand),
}

#[derive(Debug, Args)]
struct InfoInspectCommand {
    #[clap(long, help("Print internal file signature"))]
    signature: bool,
}

#[derive(Debug, Args)] #[clap(about)]
struct MetadataInspectCommand {
    #[clap(long, help("Print only the \"statsJson\" key's content as a JSON"))]
    stats: bool,

    #[clap(long, help("NOT IMPLEMENTED - Print only the values corresponding to a specific key"))]
    key: Option<String>,
}


#[derive(Debug, Args)]
struct PayloadInspectCommand {
    #[clap(long, help("Print the game's ID"))]
    id: bool,

    #[clap(long, help("Print the game's duration in seconds"))]
    duration: bool,

    #[clap(long, arg_enum, multiple_values(true), help("Print the total number of chunks or keyframes"))]
    count: Vec<SegmentType>,

    #[clap(long, help("Print the ID of the last loading chunk for the game"))]
    loadid: bool,

    #[clap(long, help("Print the ID of the first chunk after the game's start"))]
    startid: bool,

    #[clap(long, help("Print keyframe interval in seconds"))]
    interval: bool,

    #[clap(long, help("Print the file's primary encryption key"))]
    key: bool,
}

#[derive(Debug, Args)]
struct RawDataInspectCommand {
}

#[derive(Debug, Args)]
struct ExportCommand {
    #[clap(subcommand)]
    command: SubExportCommands,

    #[clap(short, long, global=true, default_value=".", help("Data export output directory"))]
    directory: std::path::PathBuf,
}

#[derive(Debug, Subcommand)]
enum SubExportCommands {
    #[clap(alias = "c", about = "Export chunks")]
    Chunk(SegmentExportCommand),

    #[clap(alias = "k", about = "Export keyframes")]
    Keyframe(SegmentExportCommand),

    #[clap(alias = "a", about = "Export everything")]
    All(FullSegmentExportCommand),
}

#[derive(Debug, Args)]
struct SegmentExportCommand {

    #[clap(long, conflicts_with("id"), help("Used to export all chunks or keyframes in the file - default to true if no chunk is configured"))]
    all: bool,

    #[clap(short, long, global = true, help("Chunk/keyframe IDs to export"))]
    id: Vec<u32>,
}

#[derive(Debug, Args)]
struct FullSegmentExportCommand {
}

#[derive(Debug, Args)]
struct AnalyzeCommand {
    #[clap(short, long, help("Which segment IDs to analyze"))]
    id: Vec<u32>,

    #[clap(long, arg_enum, default_value="stats", help("What information to look for/display"))]
    mode: AnalyzeCommandMode,

    #[clap(long, arg_enum, help("Which segment type to analyze"))]
    only: Option<SegmentType>,

    #[clap(long("type"), help("In stats mode, a specific type whose length stats should be calculated"))]
    typed: Option<usize>,

    #[clap(short('H'), long("human-readable"), help("Improve display for reading by a human"))]
    human: bool,
}

#[derive(ArgEnum, Clone, Debug)]
enum AnalyzeCommandMode {
    Bytes,
    Detail,
    Stats,
    Verify,
}

#[derive(ArgEnum, Clone, Debug, PartialEq, Eq)]
enum SegmentType {
    Chunk,
    Keyframe,
}

fn main() {
    let args = Cli::parse();
    if args.file.is_none() {
        println!("A path to a source file MUST be provided");
        std::process::exit(1);
    }
    let source_file = args.file.unwrap();
    if !source_file.exists() {
        println!("Source file does not exist: {}", source_file.display());
        std::process::exit(1);
    }

    match args.command {
        CliCommands::Get(inspect_args) => {
            match inspect_args.command {
                SubInspectCommands::Info(info_args) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    if info_args.signature {
                        println!("{:?}", data.head().signature());
                    }
                },
                SubInspectCommands::Metadata(meta_args) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    let json_metadata_string = data.metadata().unwrap();
                    if !meta_args.stats {
                        println!("{}", json_metadata_string);
                    } else {
                        let metadata = parse(json_metadata_string).unwrap();
                        println!("{}", metadata["statsJson"].as_str().unwrap());
                    }
                },
                SubInspectCommands::Payload(payload_args) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    let payload = data.payload().unwrap();
                    if payload_args.id {
                        println!("ID: {}", payload.id());
                    }
                    if payload_args.duration {
                        println!("Duration: {} ms", payload.duration());
                    }
                    for segment_type in payload_args.count {
                        match segment_type {
                            SegmentType::Chunk => {println!("ChunkCount: {}", payload.chunk_count())},
                            SegmentType::Keyframe => {println!("KeyframeCount: {}", payload.keyframe_count())},
                        }
                    }
                    if payload_args.loadid {
                        println!("LoadEndChunk: {}", payload.load_end_chunk());
                    }
                    if payload_args.startid {
                        println!("StartChunk: {}", payload.game_start_chunk());
                    }
                    if payload_args.interval {
                        println!("KeyframeInterval: {}", payload.keyframe_interval());
                    }
                    if payload_args.key {
                        println!("EncryptionKey: {}", payload.encryption_key());
                    }
                },
                SubInspectCommands::RawData(_) => {
                    eprintln!("Exported payload data inspection is not supported yet");
                    std::process::exit(1);
                },
            }
        },
        CliCommands::Export(export_args) => {
            let is_dir_valid = std::fs::metadata(&export_args.directory)
                .ok()
                .and_then(|f| if f.is_dir() {Some(())} else {None})
                .or_else(|| std::fs::create_dir(&export_args.directory).ok());
            if is_dir_valid.is_none() {
                eprintln!("Could not access nor create directory at {:?}", &export_args.directory);
                std::process::exit(1)
            }
            match export_args.command {
                SubExportCommands::Chunk(chunk_args) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    for segment in data.segment_iter().unwrap() {
                        if segment.is_chunk() && (chunk_args.all || chunk_args.id.is_empty() || chunk_args.id.contains(&segment.id())){
                            let output_file = export_args.directory.join(format!("{}-{}-Chunk.bin", data.payload().unwrap().id(), segment.id()));
                            let write_success = std::fs::write(&output_file, segment.data());
                            if write_success.is_err() {
                                eprintln!("An error occured while writing to {:?} ({})", &output_file, write_success.unwrap_err());
                                std::process::exit(1)
                            }
                        }
                    }
                },
                SubExportCommands::Keyframe(keyframe_args) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    for segment in data.segment_iter().unwrap() {
                        if segment.is_keyframe() && (keyframe_args.all || keyframe_args.id.is_empty() || keyframe_args.id.contains(&segment.id())){
                            let output_file = export_args.directory.join(format!("{}-{}-Keyframe.bin", data.payload().unwrap().id(), segment.id()));
                            let write_success = std::fs::write(&output_file, segment.data());
                            if write_success.is_err() {
                                eprintln!("An error occured while writing to {:?} ({})", &output_file, write_success.unwrap_err());
                                std::process::exit(1)
                            }
                        }
                    }
                },
                SubExportCommands::All(_) => {
                    let content = std::fs::read(source_file).unwrap();
                    let data = Rofl::from_slice(&content[..]).unwrap();
                    for segment in data.segment_iter().unwrap() {
                        let output_file = export_args.directory.join(format!(
                            "{}-{}-{}.bin", data.payload().unwrap().id(), segment.id(),
                            if segment.is_chunk() { "Chunk" } else { "Keyframe" }
                        ));
                        let write_success = std::fs::write(&output_file, segment.data());
                        if write_success.is_err() {
                            eprintln!("An error occured while writing to {:?} ({})", &output_file, write_success.unwrap_err());
                            std::process::exit(1)
                        }
                    }
                },
            }
        },
        CliCommands::Analyze(analyze_args) => {
            let content = std::fs::read(source_file).unwrap();
            let data = Rofl::from_slice(&content[..]).unwrap();
            for segment in data.segment_iter().unwrap() {
                let is_analyzed = 
                    ( // No filter is applied
                        analyze_args.id.len() == 0 && analyze_args.only.is_none()
                    ) || ( // A filter is applied and the segment is a chunk
                        segment.is_chunk()
                        && (analyze_args.id.contains(&segment.id()) || analyze_args.id.len() == 0)
                        && analyze_args.only != Some(SegmentType::Keyframe)
                    ) || ( // A filter is applied and the segment is a keyframe
                        segment.is_keyframe()
                        && (analyze_args.id.contains(&segment.id()) || analyze_args.id.len() == 0)
                        && analyze_args.only != Some(SegmentType::Chunk)
                    );
                if is_analyzed { // TODO: cleanup this code, it's a mess
                    let mut iterator = segment.section_iter().unwrap();
                    let mut last_segment: Option<GenericSection> = None;
                    let mut inventory_count = std::collections::HashMap::<usize, usize>::new();
                    let mut all_datas: Vec<Vec<u8>> = Vec::new();
                    let mut total_subdata = 0;
                    for g in iterator.by_ref() {
                        all_datas.push(g.bytes().to_vec());
                        if g.kind() == 225 || g.kind() == 209 {
                            //println!("{:?}", g.bytes());
                        }
                        if analyze_args.typed.is_none() {
                            total_subdata +=1;
                            inventory_count.insert(g.kind() as usize, inventory_count.get(&(g.kind() as usize)).unwrap_or(&0) + 1);
                        } else if Some(g.kind() as usize) == analyze_args.typed {
//                            println!("tee {:?}", g.bytes());
                            total_subdata +=1;
                            inventory_count.insert(g.len() as usize, inventory_count.get(&g.len()).unwrap_or(&0) + 1);
                        }
                        last_segment = Some(g);
                    }
                    match analyze_args.mode {
                        AnalyzeCommandMode::Bytes => println!(
                            "{} {}: {:?}",
                            if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                            segment.id(),
                            segment.data(),
                        ),
                        AnalyzeCommandMode::Detail => {
                            if !iterator.is_valid() {
                                eprintln!(
                                    "BROKE at index {} of {} {}, next bytes: {:?}",
                                    iterator.internal_index(),
                                    if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                    segment.id(),
                                    &iterator.internal_slice()[iterator.internal_index()..std::cmp::min(iterator.internal_index()+20, iterator.internal_slice().len())],
                                );
                            }
                            if analyze_args.human {
                                println!(
                                    "{} {}: [",
                                    if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                    segment.id(),
                                );
                                for data in all_datas {
                                    println!("{:?},", data);
                                }
                                if args.verbose && !iterator.is_valid() {
                                    println!(
                                        "{:?},",
                                        &iterator.internal_slice()[iterator.internal_index()..iterator.internal_slice().len()],
                                    );
                                }
                                println!("]");
                            } else {
                                println!("{}{}: {:?}", if segment.is_chunk() {"C"} else {"K"}, segment.id(), all_datas);
                            }
                        },
                        AnalyzeCommandMode::Stats => {
                            if !iterator.is_valid() {
                                eprintln!(
                                    "BROKE at index {} of {} {}, next bytes: {:?}",
                                    iterator.internal_index(),
                                    if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                    segment.id(),
                                    &iterator.internal_slice()[iterator.internal_index()..std::cmp::min(iterator.internal_index()+20, iterator.internal_slice().len())],
                                );
                            }
                            print!(
                                "{} {:#03} ({:#07}): {}",
                                if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                segment.id(),
                                segment.data().len(),
                                total_subdata,
                            );
                            if args.verbose {
                                print!(" {{");
                                let mut sorted_keys = inventory_count.keys().collect::<Vec<&usize>>();
                                sorted_keys.sort();
                                for k in sorted_keys {
                                    print!("{}: {}, ", k, inventory_count.get(k).unwrap());
                                }
                                print!("}}");
                            }
                            println!("");
                        }
                        AnalyzeCommandMode::Verify => {
                            if args.verbose && !iterator.is_valid() {
                                eprint!(
                                    "BROKE at index {} of {} {}",
                                    iterator.internal_index(),
                                    if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                    segment.id(),
                                );
                                last_segment.and_then(|g| {
                                    eprint!(", last dataset type: {} ({} bytes)",g.kind(), g.len());
                                    Some(())
                                });
                                eprintln!(
                                    ", next bytes: {:?}",
                                    &iterator.internal_slice()[iterator.internal_index()..std::cmp::min(iterator.internal_index()+20, iterator.internal_slice().len())],    
                                );
                            }
                            println!(
                                "{} {} {}",
                                if iterator.is_valid() {"SUCCESS"} else {"FAIL"},
                                if segment.is_chunk() {"Chunk"} else {"Keyframe"},
                                segment.id(),
                            )
                        },
                    }
                }
            }
            match analyze_args.mode {
                AnalyzeCommandMode::Bytes => {},
                AnalyzeCommandMode::Detail => {},
                AnalyzeCommandMode::Stats => {},
                AnalyzeCommandMode::Verify => {},
            }

        }
    }
}
