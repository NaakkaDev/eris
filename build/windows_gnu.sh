# Builds the Windows binary and zips it up with other required files
# for a nice portable solution.
#
# Requires `mingw64` directory with all the required GTK3 libs & binaries.
#
# Run in source root directory, the one with the `src` directory in it.
#

GTK_LIBRARY="$(pwd)/mingw64"
APP_DIR="$(pwd)/eris_app"
BIN_DIR="$APP_DIR/bin"

mkdir "$APP_DIR"
mkdir "$BIN_DIR"

# Copy required dll files to the bin dir
while read -r file; do cp "$GTK_LIBRARY/$file" "$BIN_DIR"; done <build/filelist.txt
# Copy required exe file to the bin dir
cp "$GTK_LIBRARY"/bin/gdbus.exe "$BIN_DIR"
# Make the share dirs inside the app dir
mkdir -p "$APP_DIR"/share/glib-2.0/schemas
mkdir -p "$APP_DIR"/share/icons
mkdir -p "$APP_DIR"/share/gtk-3.0

# Create gtk settings file
printf "[Settings]\ngtk-alternative-sort-arrows = true" > "$APP_DIR"/share/gtk-3.0/settings.ini
# Copy required schema files
cp "$GTK_LIBRARY"/share/glib-2.0/schemas/* "$APP_DIR"/share/glib-2.0/schemas
# Copy required icon files
gtk-icon-debloat -s "$GTK_LIBRARY"/share/icons -i build/iconlist.txt -o "$APP_DIR"/share/icons
# Make the lib dir inside app dir
mkdir "$APP_DIR"/lib
# Copy required lib files
cp -r "$GTK_LIBRARY"/lib/gdk-pixbuf-2.0 "$APP_DIR"/lib

# Windows build
PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_PATH="$GTK_LIBRARY/lib/pkgconfig" RUSTFLAGS="-L $GTK_LIBRARY/lib" cargo build --target=x86_64-pc-windows-gnu --bin eris --release
# Strip symbols from binary
strip target/x86_64-pc-windows-gnu/release/eris.exe
# Copy build exe file to the bin dir
cp target/x86_64-pc-windows-gnu/release/eris.exe "$BIN_DIR"
# Create Windows shortcut to the exe file
cp build/eris.exe.lnk "$APP_DIR"

zip -r eris_app.zip ./eris_app