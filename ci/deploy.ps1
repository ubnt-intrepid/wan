param(
  [parameter(mandatory)]
  [string]$pkgname,

  [string]$features
)

Remove-Item -Recurse -Force ".\$($pkgname)" -ErrorAction SilentlyContinue
cargo install --features "$($features)" --root ".\$($pkgname)"

7z a "$($pkgname).zip" ".\$($pkgname)\"
