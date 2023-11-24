#!/bin/bash

libnslversion="2.27"
debpkg="libjack-jackd2-0_1.9.12~dfsg-2_amd64.deb"
debpkg2="libc6_2.27-3ubuntu1.6_amd64.deb"

#Download building tools.
if [[ -f /usr/bin/wget ]]
then
   if [[ -f pkg2appimage--x86_64.AppImage ]]
   then
      echo "pkg2appimage--x86_64.AppImage found."
   else
      wget "https://github.com/AppImage/pkg2appimage/releases/download/continuous/pkg2appimage--x86_64.AppImage"
      #Make sure pkg2appimage is available.
      if [[ -f pkg2appimage--x86_64.AppImage ]]
      then
         echo "pkg2appimage--x86_64.AppImage found."
      else
         echo "failed to download pkg2appimage--x86_64.AppImage" && exit 1
      fi
   fi
else
   echo "please install wget." && exit 1
fi

if [[ -f appimagetool-x86_64.AppImage ]]
then 
   echo "appimagetool-x86_64.AppImage found."
else
   wget "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
   #Make sure appimagetool is available.
   if [[ -f appimagetool-x86_64.AppImage ]]
   then
      echo "appimagetool-x86_64.AppImage found."
   else
      echo "failed to download appimagetool-x86_64.AppImage" && exit 1
   fi
fi

#Make exec.
chmod +x pkg2appimage--x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

#Create BlueRecorder.yml file.
echo "create BlueRecorder.yml"
cat > BlueRecorder.yml <<EOF
app: BlueRecorder

ingredients:
  dist: bionic
  packages:
    - ffmpeg
    - x11-utils
    - gstreamer1.0-plugins-bad
    - gstreamer1.0-plugins-base
    - gstreamer1.0-plugins-good
    - gstreamer1.0-plugins-ugly
    - gstreamer1.0-libav

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
  - Comment=A simple desktop recorder for Linux systems. Built using GTK 4 and ffmpeg.
  - EOF
EOF

#Building AppImage using pkg2appimage.
echo "Building appimage directory..."
"$PWD/pkg2appimage--x86_64.AppImage" BlueRecorder.yml
rm -rf "$PWD/out"

#Installing libjack to AppDir.
if [[ -f $debpkg ]]
then
   echo "$debpkg found."
else
   apt download libjack-jackd2-0
   if [[ -f $debpkg ]]
   then
      echo "$debpkg found."
   else
      echo "failed to download $debpkg" && exit 1
   fi
fi

dpkg-deb -x "$debpkg" "$PWD"
mkdir -p "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/"
cp "usr/lib/x86_64-linux-gnu/libjack.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjack.so.0"
#cp "usr/lib/x86_64-linux-gnu/libjack.so.0.1.0" "BlueRecorder/BlueRecorder.AppDirusr/lib/x86_64-linux-gnu/libjack.so.0.1.0"
cp "usr/lib/x86_64-linux-gnu/libjacknet.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjacknet.so.0"
cp "usr/lib/x86_64-linux-gnu/libjacknet.so.0.1.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjacknet.so.0.1.0"
cp "usr/lib/x86_64-linux-gnu/libjackserver.so.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjackserver.so.0"
cp "usr/lib/x86_64-linux-gnu/libjackserver.so.0.1.0" "BlueRecorder/BlueRecorder.AppDir/usr/lib/x86_64-linux-gnu/libjackserver.so.0.1.0"

#Installing libnsl to AppDir.
if [[ -f $debpkg2 ]]
then
   echo "$debpkg2 found."
else
   apt download libc6
   if [[ -f $debpkg2 ]]
   then
      echo "$debpkg2 found."
   else
      echo "failed to download $debpkg2" && exit 1
   fi
fi

dpkg-deb -x "$debpkg2" "$PWD"
mkdir -p "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/"
cp "lib/x86_64-linux-gnu/libnsl.so.1" "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/libnsl.so.1"
cp "lib/x86_64-linux-gnu/libnsl-$libnslversion.so" "BlueRecorder/BlueRecorder.AppDir/lib/x86_64-linux-gnu/libnsl-$libnslversion.so"

#Building AppImage using appimagetool.
echo "Building appimage..."
"$PWD/appimagetool-x86_64.AppImage" "BlueRecorder/BlueRecorder.AppDir"
rm -rf "$PWD/BlueRecorder/blue-recorder"

echo "Finish building."
