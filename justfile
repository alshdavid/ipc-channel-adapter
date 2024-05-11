set windows-shell := ["pwsh", "-NoLogo", "-NoProfileLoadTime", "-Command"]

[unix]
bench message_count="100000":
  cargo build --release
  ./target/release/ipc_host_sync -b -m {{message_count}}

[windows]
bench message_count="100000":
  cargo build --release
  .\target\release\ipc_host_sync.exe -b -m {{message_count}}
