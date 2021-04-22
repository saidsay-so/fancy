# Maintainer: Musikid <musikid@outlook.com>
pkgname=fancy
pkgver=0.3.1
pkgrel=1
pkgdesc='Set of software which allows you to control your laptop fans.
It includes a service daemon to allow accessing to the embedded controller
and controlling it through D-Bus, and a CLI to send commands.'
makedepends=('rust' 'pandoc')
depends=('dbus')
optdepends=('systemd: manage the service')
arch=('i686' 'x86_64')
source=("$pkgname-$pkgver.tar.gz::https://github.com/MusiKid/$pkgname/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('ba4499af6e3b3a7afb4aa13a07142b0f873e950d0ae0785fa987cb78a099d757')
url='https://github.com/MusiKid/fancy'
license=('MPL2')

build() {
  cd "$pkgname-$pkgver"
  make
}

check() {
  cd "$pkgname-$pkgver"
  cargo test --locked --all --all-features
}

package() {
  cd "$pkgname-$pkgver"
  make install -- prefix=/usr DESTDIR="$pkgdir"
}

clean() {
  cd "$pkgname-$pkgver"
  make clean
}
