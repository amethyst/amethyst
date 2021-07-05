use std::{
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
    thread::JoinHandle,
};

use amethyst_error::Error;
use distill::daemon::AssetDaemon as AtelierAssetDaemon;
use structopt::StructOpt;
use tokio::sync::oneshot::Sender;

use crate::{prefab::PrefabImporter, simple_importer::get_source_importers};

/// Parameters to the asset daemon.
///
/// # Examples
///
/// ```bash
/// asset_daemon --db .assets_db --address "127.0.0.1:9999" assets
/// ```
#[derive(StructOpt, Debug, Clone)]
pub struct AssetDaemonArgs {
    /// Path to the asset metadata database directory.
    #[structopt(name = "db", long, parse(from_os_str), default_value = ".assets_db")]
    pub db_dir: PathBuf,
    /// Socket address for the daemon to listen for connections, e.g. "127.0.0.1:9999".
    #[structopt(
    short,
    long,
    parse(try_from_str = parse_socket_addr),
    default_value = "127.0.0.1:9999"
    )]
    pub address: SocketAddr,
    /// Directories to watch for assets.
    #[structopt(parse(from_os_str), default_value = "assets")]
    pub asset_dirs: Vec<PathBuf>,
}

impl From<AssetDaemonArgs> for AssetDaemonOpt {
    fn from(args: AssetDaemonArgs) -> AssetDaemonOpt {
        AssetDaemonOpt {
            db_dir: args.db_dir,
            address: args.address,
            asset_dirs: args.asset_dirs,
        }
    }
}

pub struct AssetDaemonOpt {
    pub db_dir: PathBuf,
    pub address: SocketAddr,
    pub asset_dirs: Vec<PathBuf>,
}

impl Default for AssetDaemonOpt {
    fn default() -> Self {
        AssetDaemonOpt {
            db_dir: ".assets_db".into(),
            address: "127.0.0.1:9999".parse().unwrap(),
            asset_dirs: vec!["assets".into()],
        }
    }
}

/// Parses a string as a socket address.
fn parse_socket_addr(s: &str) -> std::result::Result<SocketAddr, AddrParseError> {
    s.parse()
}

/// Encapsulates a Distil asset daemon.
pub struct AssetDaemon {
    state: AssetDaemonState,
}

enum AssetDaemonState {
    Initialized(InitializedDaemon),
    Started(StartedDaemon),
    Stopped,
}

struct InitializedDaemon {
    opt: AssetDaemonOpt,
}
struct StartedDaemon {
    shutdown: Option<Sender<bool>>,
    join_handle: Option<JoinHandle<()>>,
}

impl AssetDaemon {
    /// Returns an `AssetDaemon` initialized with asset directories.
    ///
    /// # Arguments
    ///
    /// * `asset_dirs` - The directories to load assets from.
    #[must_use]
    pub fn new(asset_dirs: Vec<PathBuf>) -> Self {
        let opt = AssetDaemonOpt {
            db_dir: ".assets_db".into(),
            address: "127.0.0.1:9999".parse().unwrap(),
            asset_dirs,
        };
        AssetDaemon {
            state: AssetDaemonState::Initialized(InitializedDaemon { opt }),
        }
    }
    /// Starts the asset daemon on a new thread.
    pub fn start_on_new_thread(&mut self) {
        if let AssetDaemonState::Initialized(daemon) = &self.state {
            self.state = daemon.start_on_new_thread();
        }
    }
    /// Stops the asset daemon and blocks until the asset daemon thread joins the callers thread.
    pub fn stop_and_join(&mut self) {
        if let AssetDaemonState::Started(daemon) = &mut self.state {
            self.state = daemon.stop_and_join();
        }
    }
}

impl InitializedDaemon {
    fn start_on_new_thread(&self) -> AssetDaemonState {
        let (join_handle, shutdown) = AtelierAssetDaemon::default()
            .with_importers_boxed(get_source_importers())
            .with_importer("prefab", PrefabImporter::default())
            .with_db_path(self.opt.db_dir.clone())
            .with_address(self.opt.address)
            .with_asset_dirs(self.opt.asset_dirs.clone())
            .run();
        AssetDaemonState::Started(StartedDaemon {
            shutdown: Some(shutdown),
            join_handle: Some(join_handle),
        })
    }
}

impl StartedDaemon {
    fn stop_and_join(&mut self) -> AssetDaemonState {
        self.shutdown
            .take()
            .ok_or_else(|| Error::from_string("Shutdown Sender not present"))
            .and_then(|s| {
                s.send(true)
                    .map_err(|_| Error::from_string("failure sending shutdown to assetdaemon"))
            })
            .and_then(|_| {
                self.join_handle
                    .take()
                    .ok_or_else(|| Error::from_string("JoinHandle not present"))
            })
            .and_then(|h| {
                h.join()
                    .map_err(|_| Error::from_string("Failure joining AssetDaemon thread"))
            })
            .map_or(AssetDaemonState::Stopped, |_| AssetDaemonState::Stopped)
    }
}
