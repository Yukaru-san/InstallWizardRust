use std::{
    fs::File,
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use installwizard_core::{Cancellation, Error, Package, Result, Target, inspect_package};
use installwizard_native::{ExportOptions, Overwrite, Project, Source, build, export_web, extract};

#[derive(Parser)]
#[command(version, author, about = "Create and extract portable file installers")]
struct Arguments {
    #[arg(long, conflicts_with = "gui")]
    cli: bool,
    /// Print bundled third-party license notices.
    #[arg(long)]
    licenses: bool,
    #[arg(long, conflicts_with = "cli")]
    gui: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Create an installer or a static web wizard.
    Create {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        name: Option<String>,
        /// Source mapping: SOURCE=relative/destination; repeat for more sources.
        #[arg(long = "source", value_parser = parse_source)]
        sources: Vec<Source>,
        #[arg(long)]
        icon: Option<PathBuf>,
        /// windows-x64, windows-arm64, linux-x64, linux-arm64, macos-x64, macos-arm64; repeat or use all.
        #[arg(long = "target")]
        targets: Vec<String>,
        #[arg(long, default_value = "output")]
        output: PathBuf,
        #[arg(long)]
        templates: Option<PathBuf>,
        #[arg(long)]
        web: bool,
        #[arg(long)]
        overwrite: bool,
    },
    /// Extract the embedded package (or --package) to a folder.
    Extract {
        #[arg(long)]
        destination: Option<PathBuf>,
        #[arg(long)]
        package: Option<PathBuf>,
        #[arg(long)]
        overwrite: bool,
    },
    /// Display the embedded package's file list as JSON.
    List {
        #[arg(long)]
        package: Option<PathBuf>,
    },
    /// Verify every embedded file's length and SHA-256.
    Verify {
        #[arg(long)]
        package: Option<PathBuf>,
    },
}

fn parse_source(value: &str) -> std::result::Result<Source, String> {
    let (source, destination) = value
        .split_once('=')
        .ok_or("Use --source SOURCE=relative/destination")?;
    if source.is_empty() {
        return Err("Source path is empty".into());
    }
    Ok(Source {
        source: source.into(),
        destination: destination.into(),
    })
}

/// Locate data in this executable, or in the macOS bundle's Resources directory.
pub fn own_package() -> Result<Option<(PathBuf, Package)>> {
    let path = std::env::current_exe()?;
    if let Some(package) = inspect_package(&mut File::open(&path)?)? {
        return Ok(Some((path, package)));
    }
    #[cfg(target_os = "macos")]
    if let Some(contents) = path.parent().and_then(|path| path.parent()) {
        let resource = contents.join("Resources/payload.iwpkg");
        if resource.is_file() {
            let package = inspect_package(&mut File::open(&resource)?)?
                .ok_or_else(|| Error::Invalid("Invalid bundle payload".into()))?;
            return Ok(Some((resource, package)));
        }
    }
    Ok(None)
}

static CLI_CANCEL: std::sync::OnceLock<Cancellation> = std::sync::OnceLock::new();
static CLI_WORKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn cancellation() -> Cancellation {
    CLI_WORKING.store(true, std::sync::atomic::Ordering::Relaxed);
    CLI_CANCEL.get_or_init(Cancellation::default).clone()
}

pub fn run() -> Result<()> {
    ctrlc::set_handler(|| {
        if CLI_WORKING.load(std::sync::atomic::Ordering::Relaxed) {
            CLI_CANCEL.get_or_init(Cancellation::default).cancel();
        } else {
            std::process::exit(130);
        }
    })
    .map_err(|error| Error::Invalid(format!("Could not install interrupt handler: {error}")))?;
    #[cfg(target_os = "windows")]
    let parent_terminal = unsafe {
        use windows_sys::Win32::System::Console::{
            ATTACH_PARENT_PROCESS, AttachConsole, GetConsoleWindow,
        };
        !GetConsoleWindow().is_null() || AttachConsole(ATTACH_PARENT_PROCESS) != 0
    };
    #[cfg(not(target_os = "windows"))]
    let parent_terminal = io::stdin().is_terminal() || io::stdout().is_terminal();
    let args = Arguments::parse();
    if args.licenses {
        print!(
            "{}",
            include_str!("../../../assets/THIRD_PARTY_NOTICES.txt")
        );
        return Ok(());
    }
    if args.gui {
        return crate::native_ui::launch(own_package()?);
    }
    let own = own_package()?;
    let Some(command) = args.command else {
        if !args.cli && !parent_terminal {
            return crate::native_ui::launch(own);
        }
        if let Some((path, package)) = own {
            return extract_command(path, package, None, false);
        }
        return interactive_creator();
    };
    match command {
        Command::Create {
            project,
            name,
            sources,
            icon,
            targets,
            output,
            templates,
            web,
            overwrite,
        } => {
            if own.is_some() {
                return Err(Error::Invalid(
                    "This executable is an installer. Use the creator to build packages.".into(),
                ));
            }
            let mut project = project
                .map(|path| Project::load(&path))
                .transpose()?
                .unwrap_or_default();
            if let Some(name) = name {
                project.name = name;
            }
            if !sources.is_empty() {
                project.sources = sources;
            }
            if icon.is_some() {
                project.icon = icon;
            }
            if !targets.is_empty() {
                project.targets = if targets.iter().any(|target| target == "all") {
                    Target::ALL.to_vec()
                } else {
                    targets
                        .iter()
                        .map(|target| Target::parse(target))
                        .collect::<Result<_>>()?
                };
            }
            let options = ExportOptions {
                output,
                templates,
                overwrite,
            };
            let outputs = if web {
                vec![export_web(&project, &options, &cancellation(), |_| {})?]
            } else {
                build(&project, &options, &cancellation(), |_| {})?
            };
            for output in outputs {
                println!("{}", output.display());
            }
        }
        Command::Extract {
            destination,
            package,
            overwrite,
        } => {
            let (path, package) = load_package(package, own)?;
            extract_command(path, package, destination, overwrite)?;
        }
        Command::List { package } => {
            let (_, package) = load_package(package, own)?;
            serde_json::to_writer_pretty(io::stdout().lock(), &package.manifest)?;
            println!();
        }
        Command::Verify { package } => {
            let (path, package) = load_package(package, own)?;
            package.verify(&mut File::open(path)?, &cancellation(), |_| {})?;
            println!("Integrity verified.");
        }
    }
    Ok(())
}

fn load_package(
    path: Option<PathBuf>,
    own: Option<(PathBuf, Package)>,
) -> Result<(PathBuf, Package)> {
    if let Some(path) = path {
        let package = inspect_package(&mut File::open(&path)?)?
            .ok_or_else(|| Error::Invalid("No InstallWizard package found".into()))?;
        Ok((path, package))
    } else {
        own.ok_or_else(|| Error::Invalid("No embedded package. Supply --package <file>.".into()))
    }
}

fn prompt(label: &str) -> Result<String> {
    if !io::stdin().is_terminal() {
        return Err(Error::Invalid(
            "Missing required arguments in non-interactive mode. See --help.".into(),
        ));
    }
    eprint!("{label}: ");
    io::stderr().flush()?;
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return Err(Error::Cancelled);
    }
    Ok(input.trim().to_string())
}

fn extract_command(
    path: PathBuf,
    package: Package,
    destination: Option<PathBuf>,
    overwrite: bool,
) -> Result<()> {
    eprintln!(
        "{} — {} entries",
        package.manifest.name,
        package.manifest.entries.len()
    );
    let destination = match destination {
        Some(path) => path,
        None => PathBuf::from(prompt("Destination folder")?),
    };
    if destination.as_os_str().is_empty() {
        return Err(Error::Invalid("Destination is empty".into()));
    }
    extract(
        &mut File::open(path)?,
        &package,
        &destination,
        if overwrite {
            Overwrite::Replace
        } else {
            Overwrite::Refuse
        },
        &cancellation(),
        |_| {},
    )?;
    println!("Extracted to {}", destination.display());
    Ok(())
}

fn interactive_creator() -> Result<()> {
    let name = prompt("Package name")?;
    let mut sources = Vec::new();
    loop {
        let source = prompt("File or folder (empty to finish)")?;
        if source.is_empty() {
            break;
        }
        let source = PathBuf::from(source);
        let default = source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("files");
        let destination = prompt(&format!("Package destination [{default}]"))?;
        sources.push(Source {
            destination: if destination.is_empty() {
                default.into()
            } else {
                destination
            },
            source,
        });
    }
    let icon = prompt("Icon PNG/ICO (optional)")?;
    let mut project = Project {
        name,
        sources,
        icon: if icon.is_empty() {
            None
        } else {
            Some(icon.into())
        },
        ..Project::default()
    };
    let target = prompt("Targets, comma-separated; empty = this platform; all = all platforms")?;
    if !target.is_empty() {
        project.targets = if target == "all" {
            Target::ALL.to_vec()
        } else {
            target
                .split(',')
                .map(|s| Target::parse(s.trim()))
                .collect::<Result<_>>()?
        };
    }
    let output = prompt("Output folder [output]")?;
    let templates = prompt("Runtime template folder (optional)")?;
    let options = ExportOptions {
        output: if output.is_empty() {
            "output".into()
        } else {
            output.into()
        },
        templates: if templates.is_empty() {
            None
        } else {
            Some(templates.into())
        },
        overwrite: false,
    };
    for output in build(&project, &options, &cancellation(), |_| {})? {
        println!("{}", output.display());
    }
    Ok(())
}
