SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
cd $SCRIPTPATH

hyperfine --min-runs 3 --prepare 'cargo clean' 'cargo build'

hyperfine --min-runs 3 --prepare 'cargo clean' 'cargo build --release'

# Does not show a large difference. Probably not representative of real world lto impact.
# CARGO_PROFILE_release_LTO=thin hyperfine --prepare 'cargo clean' 'cargo build --release'
