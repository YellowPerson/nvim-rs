use nvim_rs::{create, DefaultHandler};

use tokio;

use std::{
  thread::sleep,
  time::{Duration, Instant},
};

use std::process::Command;

#[cfg(unix)]
use tempdir::TempDir;
#[cfg(unix)]
use std::path::Path;

const NVIMPATH: &str = "neovim/build/bin/nvim";
const HOST: &str = "0.0.0.0";
const PORT: u16 = 6666;

#[tokio::test]
async fn can_connect_via_tcp() {
  let listen = HOST.to_string() + ":" + &PORT.to_string();

  let _child = Command::new(NVIMPATH)
    .args(&["-u", "NONE", "--embed", "--headless", "--listen", &listen])
    .spawn()
    .expect("Cannot start neovim");

  // wait at most 1 second for neovim to start and create the tcp socket
  let start = Instant::now();

  let (nvim, _io_handle) = loop {
    sleep(Duration::from_millis(100));

    let handler = DefaultHandler::new();
    if let Ok(r) = create::new_tcp(&listen, handler).await {
      break r;
    } else {
      if Duration::from_secs(1) <= start.elapsed() {
        panic!("Unable to connect to neovim via tcp at {}", listen);
      }
    }
  };

  let servername = nvim
    .get_vvar("servername")
    .await
    .expect("Error retrieving servername from neovim");

  assert_eq!(&listen, servername.as_str().unwrap());
}

#[cfg(unix)]
#[tokio::test]
async fn can_connect_via_unix_socket() {
  let dir = TempDir::new("neovim-lib.test")
    .expect("Cannot create temporary directory for test.");

  let socket_path = dir.path().join("unix_socket");

  let _child = Command::new(NVIMPATH)
    .args(&["-u", "NONE", "--embed", "--headless"])
    .env("NVIM_LISTEN_ADDRESS", &socket_path)
    .spawn()
    .expect("Cannot start neovim");

  // wait at most 1 second for neovim to start and create the socket
  {
    let start = Instant::now();
    let one_second = Duration::from_secs(1);
    loop {
      sleep(Duration::from_millis(100));

      if let Ok(_) = std::fs::metadata(&socket_path) {
        break;
      }

      if one_second <= start.elapsed() {
        panic!(format!("neovim socket not found at '{:?}'", &socket_path));
      }
    }
  }

  let handler = DefaultHandler::new();
  let (nvim, _io_handle) = create::new_unix_socket(&socket_path, handler)
    .await
    .expect(&format!(
      "Unable to connect to neovim's unix socket at {:?}",
      &socket_path
    ));

  let servername = nvim
    .get_vvar("servername")
    .await
    .expect("Error retrieving servername from neovim");

  let servername = servername.as_str().unwrap();

  assert_eq!(socket_path, Path::new(servername));
}
