Name:          blue-recorder
Version:       1.3.2
Release:       1%{?dist}
Summary:       A simple desktop recorder for Linux systems.
License:       GPL-3.0-or-later
URL:           https://github.com/xlmnxp/blue-recorder
VCS:           https://github.com/xlmnxp/blue-recorder.git
Source:        blue-recorder-%{version}.tar


Requires: ffmpeg
Requires: libappindicator-gtk3
Requires: gtk3
Requires: glib2
Requires: xwininfo
Requires: pipewire-pulseaudio

BuildRequires: rust
BuildRequires: cargo
BuildRequires: clang
BuildRequires: gettext
BuildRequires: libappindicator-gtk3-devel


%description
A simple desktop recorder for Linux systems. Built using Rust, GTK+ 3 and ffmpeg. It supports recording audio and video on almost all Linux interfaces with support for Wayland display server on GNOME session.

%global debug_package %{nil}
%prep
%setup -q -n blue-recorder-%{version}

%build
cargo build --release

%install
cat > blue-recorder <<EOF
#!/bin/bash
exec /opt/blue-recorder/blue-recorder
EOF
desktop-file-edit --set-icon=blue-recorder data/blue-recorder.desktop
install -p -D -m755 blue-recorder                    %{buildroot}%{_bindir}/blue-recorder
install -p -D -m644 data/blue-recorder.desktop       %{buildroot}%{_datadir}/applications/blue-recorder.desktop
install -p -D -m644 data/blue-recorder.svg           %{buildroot}%{_datadir}/pixmaps/blue-recorder.svg
install -p -D -m644 interfaces/main.ui               %{buildroot}/opt/blue-recorder/interfaces/main.ui
cp -r data                                           %{buildroot}/opt/blue-recorder/data
cp -r po                                             %{buildroot}/opt/blue-recorder/po
install -p -D -m755 target/release/blue-recorder     %{buildroot}/opt/blue-recorder/blue-recorder


%check
desktop-file-validate %{buildroot}%{_datadir}/applications/blue-recorder.desktop

%files
%{_bindir}/blue-recorder
%{_datadir}/applications/blue-recorder.desktop
%{_datadir}/pixmaps/blue-recorder.svg
%dir /opt/blue-recorder/
/opt/blue-recorder/interfaces/main.ui
/opt/blue-recorder/data
/opt/blue-recorder/po
/opt/blue-recorder/blue-recorder
