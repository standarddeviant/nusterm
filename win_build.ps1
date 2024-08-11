
$ghash = git rev-parse --short HEAD
echo "ghash = $ghash"

cargo build --release --target x86_64-pc-windows-msvc


$dst = ".\bin\nusterm_${ghash}_x86_64-msvc.exe"
$src = ".\target\x86_64-pc-windows-msvc\release\nusterm.exe"
if (Test-Path -Path $src) {
  Copy-Item $src -Destination $dst
}

