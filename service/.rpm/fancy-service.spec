# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

Name: fancy-service
Summary: This package provides the service for Fancy, a set of software to control laptop fans.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MPLv2.0
Group: System Environment/Daemons
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root
BuildRequires: systemd-rpm-macros

%systemd_requires
Requires: dbus

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%post
%systemd_post fancy.service fancy-sleep.service

%preun
%systemd_preun fancy.service fancy-sleep.service

%postun
%systemd_postun_with_restart fancy.service fancy-sleep.service

%files
%defattr(-,root,root,-)
%{_sbindir}/*
%{_unitdir}/fancy.service
%{_unitdir}/fancy-sleep.service
%{_sysconfdir}/fancy/configs
%{_sysconfdir}/dbus-1/system.d/com.musikid.fancy.conf
%{_datadir}/dbus-1/services/com.musikid.fancy.service
