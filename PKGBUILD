# Maintainer: Musikid <musikid@outlook.com>
pkgname=fancy
pkgver=0.3.1
pkgrel=1
pkgdesc='Set of software which allows you to control your laptop fans.
It includes a service daemon to allow accessing to the embedded controller
and controlling it through D-Bus, and a CLI to send commands.'
makedepends=('rust>=1.48' 'pandoc')
depends=('dbus')
optdepends=('systemd: manage the service')
arch=('i686' 'x86_64')
source=("$pkgname-$pkgver.tar.gz::https://github.com/MusiKid/$pkgname/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('d2917cba939495f880017f7cbe80df5dea1b19f5d9355b7d8b7564ec1aee3af1')
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
