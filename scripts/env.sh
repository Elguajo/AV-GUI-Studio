# Points this Git Bash session at the tools scripts/bootstrap.ps1 installed into .tools/.
# Source it from anywhere:  source scripts/env.sh
_gs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ ! -x "$_gs_root/.tools/cargo/bin/cargo.exe" ]; then
    echo "env.sh: no .tools in this checkout yet: run scripts/bootstrap.ps1 first" >&2
else
    export RUSTUP_HOME="$_gs_root/.tools/rustup"
    export CARGO_HOME="$_gs_root/.tools/cargo"
    export PATH="$_gs_root/.tools/cargo/bin:$_gs_root/.tools/mingw64/bin:$PATH"
fi
unset _gs_root
