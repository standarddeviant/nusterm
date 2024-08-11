
$ghash = git rev-parse --short HEAD
Write-Output "ghash = $ghash"

cargo build --release --target x86_64-unknown-linux-gnu

$dst = "./nusterm_${ghash}_x86_64-linux-gnu"
$src = "./target/x86_64-unknown-linux-gnu/release/nusterm"
if (Test-Path -Path $src)
{
  Copy-Item $src -Destination $dst
}

