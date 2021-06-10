#!/bin/bash

#dist=bionic change line 40+62.
pkg2appimageVersion="1806"
#AppImageKitVersion="12"
libnslversion="2.27"
debpkg="libjack-jackd2-0_1.9.12~dfsg-2_amd64.deb"
debpkg2="libc6_2.27-3ubuntu1.2_amd64.deb"

#Download building tools.
if [[ -f /usr/bin/wget ]]
then
   if [[ -f pkg2appimage-1806-x86_64.AppImage ]]
   then
      echo "pkg2appimage-$pkg2appimageVersion-x86_64.AppImage found"
   else
      wget "https://github.com/AppImage/pkg2appimage/releases/download/continuous/pkg2appimage-$pkg2appimageVersion-x86_64.AppImage"
   fi
else
   echo "please install wget" && exit
fi

if [[ -f appimagetool-x86_64.AppImage ]]
then 
   echo "appimagetool-x86_64.AppImage found"
else
   #wget "https://github.com/AppImage/AppImageKit/releases/download/$AppImageKitVersion/appimagetool-x86_64.AppImage"
   wget "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
fi

#Make exec.
chmod +x "pkg2appimage-$pkg2appimageVersion-x86_64.AppImage"
chmod +x appimagetool-x86_64.AppImage

#Create BlueRecorder.yml file.
cat > BlueRecorder.yml <<EOF
app: BlueRecorder

ingredients:
  dist: bionic
  packages:
    - ffmpeg
    - libappindicator3-1
    - x11-utils
    - pulseaudio

  script:
    - mkdir -p BlueRecorder.AppDir/usr/bin
    - git clone https://github.com/xlmnxp/blue-recorder.git
    - cd blue-recorder
    - cargo build --release
    - cd ..
    - cp -r "blue-recorder/po/" "BlueRecorder.AppDir/usr/bin/po/"
    - cp -r "blue-recorder/interfaces/" "BlueRecorder.AppDir/usr/bin/interfaces/"
    - cp -r "blue-recorder/data/" "BlueRecorder.AppDir/usr/bin/data/"
    - cp "blue-recorder/target/release/blue-recorder" "BlueRecorder.AppDir/usr/bin/blue-recorder"
    - cd BlueRecorder.AppDir
    - ln -s "usr/bin/data/blue-recorder.svg" "blue-recorder.svg"
  

  sources:
    - deb http://ly.archive.ubuntu.com/ubuntu/ bionic main universe

script:
  - cat > blue-recorder.desktop <<EOF
  - [Desktop Entry]
  - Type=Application
  - Name=Blue Recorder
  - Icon=blue-recorder
  - Exec=blue-recorder
  - Categories=AudioVideo;GTK;
  - Comment=A simple desktop recorder for Linux systems. Built using GTK+ 3 and ffmpeg.
  - EOF
EOF

#Building AppImage using pkg2appimage.
"$PWD/pkg2appimage-$pkg2appimageVersion-x86_64.AppImage" BlueRecorder.yml
rm -rf "$PWD/out"

#Installing libjack to AppDir.
if [[ -f $debpkg ]]
then
   echo "$debpkg was found"
else
   wget "http://mirrors.kernel.org/ubuntu/pool/main/j/jackd2/$debpkg"
   dpkg-deb -x "$debpkg" "$PWD"
fi
mkdir -p "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/"
cp "usr/lib/x86_64-linux-gnu/libjack.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjack.so.0"
cp "usr/lib/x86_64-linux-gnu/libjack.so.0.1.0" "BlueRecorder/BlueRecorder.AppDirusr/lib/x86_64-linux-gnu/libjack.so.0.1.0"
cp "usr/lib/x86_64-linux-gnu/libjacknet.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjacknet.so.0"
cp "usr/lib/x86_64-linux-gnu/libjacknet.so.0.1.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjacknet.so.0.1.0"
cp "usr/lib/x86_64-linux-gnu/libjackserver.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjackserver.so.0"
cp "usr/lib/x86_64-linux-gnu/libjackserver.so.0.1.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjackserver.so.0.1.0"

#Installing libnsl to AppDir.
if [[ -f $debpkg2 ]]
then
   echo "$debpkg2 found"
else
   wget "http://security.ubuntu.com/ubuntu/pool/main/g/glibc/$debpkg2"
   dpkg-deb -x "$debpkg2" "$PWD"
fi
mkdir -p "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/"
cp "lib/x86_64-linux-gnu/libnsl.so.1" "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/libnsl.so.1"
cp "lib/x86_64-linux-gnu/libnsl-$libnslversion.so" "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/libnsl-$libnslversion.so"

#Building AppImage using appimagetool.
"$PWD/appimagetool-x86_64.AppImage" "BlueRecorder/BlueRecorder.AppDir"
rm -rf "$PWD/BlueRecorder/blue-recorder"


