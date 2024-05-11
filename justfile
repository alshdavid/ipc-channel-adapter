set windows-shell := ["pwsh", "-NoLogo", "-NoProfileLoadTime", "-Command"]

[unix]
bench message_count="100000":
  cargo build --release
  ./target/release/ipc_host_sync -b -m {{message_count}}

[windows]
bench message_count="100000":
  cargo build --release
  .\target\release\ipc_host_sync.exe -b -m {{message_count}}

[unix]
bench-async message_count="100000":
  cargo build --release
  ./target/release/ipc_host_asynch -b -m {{message_count}}

[windows]
bench-async message_count="100000":
  cargo build --release
  .\target\release\ipc_host_asynch.exe -b -m {{message_count}}

