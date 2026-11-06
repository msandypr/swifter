use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwifterAppConfig {
    pub app_name: String,
    pub app_slug: String,
    pub target_exe_relative: String,
    pub backend: String, // "DXVK" | "D3DMetal" | "Vanilla"
    pub enable_msync: bool,
    pub enable_esync: bool,
    #[serde(default = "default_retina")]
    pub enable_retina: bool,
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    #[serde(default)]
    pub launch_args: String,
    #[serde(default)]
    pub custom_env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub wine_custom_path: Option<String>,
}

fn default_retina() -> bool {
    true
}

fn default_dpi() -> u32 {
    144
}

/// Generates the POSIX shell script content for Contents/MacOS/launcher
pub fn generate_launcher_script(config: &SwifterAppConfig) -> String {
    let mut script = String::from("#!/bin/bash\nset -e\n\n");
    script.push_str("# Swifter Native Application Launcher\n");
    script.push_str("CONTENTS_DIR=\"$(cd \"$(dirname \"$0\")/..\" && pwd)\"\n");
    script.push_str("RESOURCES_DIR=\"$CONTENTS_DIR/Resources\"\n");
    script.push_str("PAYLOAD_DIR=\"$RESOURCES_DIR/game_payload\"\n");
    script.push_str(&format!("APP_SLUG=\"{}\"\n", config.app_slug));
    script.push_str(&format!("APP_NAME=\"{}\"\n", config.app_name));
    script.push_str(&format!("EXE_REL=\"{}\"\n", config.target_exe_relative));
    script.push_str(&format!("BACKEND=\"{}\"\n", config.backend));
    script.push_str(&format!("ENABLE_MSYNC={}\n", if config.enable_msync { "1" } else { "0" }));
    script.push_str(&format!("ENABLE_ESYNC={}\n", if config.enable_esync { "1" } else { "0" }));
    script.push_str(&format!("ENABLE_RETINA={}\n", if config.enable_retina { "1" } else { "0" }));
    let dpi_val = if config.dpi == 0 { 144 } else { config.dpi };
    script.push_str(&format!("DPI={}\n", dpi_val));
    script.push_str(&format!("LAUNCH_ARGS=\"{}\"\n", config.launch_args.replace('"', "\\\"")));

    // Prefix path
    script.push_str("\n# Micro-Prefix location & Architecture\n");
    script.push_str("export WINEPREFIX=\"$HOME/Library/Application Support/Swifter/prefixes/$APP_SLUG\"\n");
    script.push_str("export WINEARCH=win64\n");
    script.push_str("mkdir -p \"$WINEPREFIX\"\n\n");

    // Wine binary search
    script.push_str(r#"# Wine runtime discovery
WINE_BIN=""

# 1. Check custom path if configured
if [ -n "" ]; then
    WINE_BIN=""
fi

# 2. Check Swifter bundled runtime
if [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/Swifter/runtimes/wine/bin/wine64" ]; then
    WINE_BIN="$HOME/Library/Application Support/Swifter/runtimes/wine/bin/wine64"
elif [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/Swifter/runtimes/wine/bin/wine" ]; then
    WINE_BIN="$HOME/Library/Application Support/Swifter/runtimes/wine/bin/wine"
fi

# 3. Check Whisky runtime
if [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/bin/wine64" ]; then
    WINE_BIN="$HOME/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/bin/wine64"
elif [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/bin/wine" ]; then
    WINE_BIN="$HOME/Library/Application Support/com.isaacmarovitz.Whisky/Libraries/Wine/bin/wine"
elif [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/Whisky/Libraries/Wine/bin/wine64" ]; then
    WINE_BIN="$HOME/Library/Application Support/Whisky/Libraries/Wine/bin/wine64"
elif [ -z "$WINE_BIN" ] && [ -x "$HOME/Library/Application Support/Whisky/Libraries/Wine/bin/wine" ]; then
    WINE_BIN="$HOME/Library/Application Support/Whisky/Libraries/Wine/bin/wine"
fi

# 4. Check Homebrew Wine (Apple Silicon & Intel)
if [ -z "$WINE_BIN" ] && [ -x "/opt/homebrew/bin/wine64" ]; then
    WINE_BIN="/opt/homebrew/bin/wine64"
elif [ -z "$WINE_BIN" ] && [ -x "/opt/homebrew/bin/wine" ]; then
    WINE_BIN="/opt/homebrew/bin/wine"
elif [ -z "$WINE_BIN" ] && [ -x "/usr/local/bin/wine64" ]; then
    WINE_BIN="/usr/local/bin/wine64"
elif [ -z "$WINE_BIN" ] && [ -x "/usr/local/bin/wine" ]; then
    WINE_BIN="/usr/local/bin/wine"
fi

# 5. Check CrossOver runtime
if [ -z "$WINE_BIN" ] && [ -x "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64" ]; then
    WINE_BIN="/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64"
fi

# 6. Check system PATH
if [ -z "$WINE_BIN" ]; then
    if command -v wine64 >/dev/null 2>&1; then
        WINE_BIN="$(command -v wine64)"
    elif command -v wine >/dev/null 2>&1; then
        WINE_BIN="$(command -v wine)"
    fi
fi

if [ -z "$WINE_BIN" ]; then
    osascript -e 'display alert "Wine Runtime Not Found" message "Swifter needs a Wine runtime to run this application.\n\nPlease install Wine in Swifter Settings." as critical'
    exit 1
fi

# OpenGL over Vulkan / Metal translation flags
export MESA_LOADER_DRIVER_OVERRIDE=zink
export GALLIUM_DRIVER=zink
export MESA_GL_VERSION_OVERRIDE=4.6
export MESA_GLSL_VERSION_OVERRIDE=460

"#);

    // Backend environment variables
    script.push_str(r#"# Translation backend configuration
export WINEDEBUG="fixme-all,err-all,+err"

if [ "$ENABLE_MSYNC" = "1" ]; then
    export WINEMSYNC=1
fi

if [ "$ENABLE_ESYNC" = "1" ]; then
    export WINEESYNC=1
fi

if [ "$BACKEND" = "D3DMetal" ]; then
    export WINEESYNC=1
    export WINEMSYNC=1
    export WINEDLLOVERRIDES="d3d12=n,b;d3d11=n,b;dxgi=b,n;${WINEDLLOVERRIDES}"
elif [ "$BACKEND" = "DXVK" ]; then
    export WINEDLLOVERRIDES="d3d11=n,b;d3d10core=n,b;d3d9=n,b;dxgi=b,n;${WINEDLLOVERRIDES}"
fi
export WINEDLLOVERRIDES="opengl32=n,b;${WINEDLLOVERRIDES}"

if [ "$ENABLE_RETINA" = "1" ]; then
    export WINE_ENABLE_RETINA=1
else
    export WINE_ENABLE_RETINA=0
fi

# Synchronize Wine prefix LogPixels registry with configured DPI
if [ -f "$WINEPREFIX/user.reg" ] && [ -n "$DPI" ]; then
    HEX_DPI=$(printf "%08x" "$DPI")
    if ! grep -q "LogPixels\"=dword:$HEX_DPI" "$WINEPREFIX/user.reg" 2>/dev/null; then
        sed -i '' -e "s/\"LogPixels\"=dword:[0-9a-fA-F]*/\"LogPixels\"=dword:$HEX_DPI/g" "$WINEPREFIX/user.reg" 2>/dev/null || true
    fi
fi
"#);

    // Custom environment variables from config
    for (k, v) in &config.custom_env {
        script.push_str(&format!("export {}=\"{}\"\n", k, v));
    }

    // Launch execution
    script.push_str(r#"
# If config file specifies target_exe_relative dynamically, read it
if [ -f "$RESOURCES_DIR/swifter.config.json" ]; then
    CFG_EXE="$(grep -o '"target_exe_relative"[^,]*' "$RESOURCES_DIR/swifter.config.json" | cut -d'"' -f4)"
    if [ -n "$CFG_EXE" ]; then
        EXE_REL="$CFG_EXE"
    fi
fi

# Resolve executable path (checks game payload, prefix root, prefix drive_c, or wine-shared)
TARGET_EXE=""

if [ -f "$PAYLOAD_DIR/$EXE_REL" ]; then
    TARGET_EXE="$PAYLOAD_DIR/$EXE_REL"
elif [ -f "$WINEPREFIX/$EXE_REL" ]; then
    TARGET_EXE="$WINEPREFIX/$EXE_REL"
elif [ -f "$WINEPREFIX/drive_c/$EXE_REL" ]; then
    TARGET_EXE="$WINEPREFIX/drive_c/$EXE_REL"
elif [ -f "$HOME/Library/Application Support/Swifter/runtimes/wine-shared/$EXE_REL" ]; then
    TARGET_EXE="$HOME/Library/Application Support/Swifter/runtimes/wine-shared/$EXE_REL"
elif [ -f "$HOME/Library/Application Support/Swifter/runtimes/wine-shared/drive_c/$EXE_REL" ]; then
    TARGET_EXE="$HOME/Library/Application Support/Swifter/runtimes/wine-shared/drive_c/$EXE_REL"
elif [ -f "$EXE_REL" ]; then
    TARGET_EXE="$EXE_REL"
fi

# If TARGET_EXE is still an installer or not found, auto-discover installed binary in Program Files or AppData
if [ -z "$TARGET_EXE" ] || echo "$EXE_REL" | grep -qiE "setup|install"; then
    DISCOVERED="$(find "$WINEPREFIX/drive_c/Program Files" "$WINEPREFIX/drive_c/Program Files (x86)" "$WINEPREFIX/drive_c/users" "$HOME/Library/Application Support/Swifter/runtimes/wine-shared/drive_c/Program Files" "$HOME/Library/Application Support/Swifter/runtimes/wine-shared/drive_c/Program Files (x86)" -maxdepth 8 -iname "*.exe" 2>/dev/null | grep -viE "unins|setup|install|crash|vcredist|updater|dotnet" | head -n 1)"
    if [ -n "$DISCOVERED" ] && [ -f "$DISCOVERED" ]; then
        TARGET_EXE="$DISCOVERED"
    fi
fi

if [ -z "$TARGET_EXE" ] || [ ! -f "$TARGET_EXE" ]; then
    osascript -e "display alert \"Executable Missing\" message \"Could not find executable: $EXE_REL\nSearched in payload, prefix, and AppData.\" as critical"
    exit 1
fi

# Set working directory to executable folder so assets load relatively
cd "$(dirname "$TARGET_EXE")"

# Check dynamic launch arguments from swifter.config.json if present
if [ -f "$RESOURCES_DIR/swifter.config.json" ]; then
    DYNAMIC_ARGS="$(grep -o '"launch_args"[^,]*' "$RESOURCES_DIR/swifter.config.json" | head -n 1 | cut -d'"' -f4 || echo "")"
    if [ -n "$DYNAMIC_ARGS" ]; then
        LAUNCH_ARGS="$DYNAMIC_ARGS"
    fi
fi

# Locate wineserver binary to accurately monitor prefix session lifecycle
WINESERVER_BIN="$(dirname "$WINE_BIN")/wineserver"
if [ ! -x "$WINESERVER_BIN" ]; then
    WINESERVER_BIN="$(command -v wineserver 2>/dev/null || echo "")"
fi

# Setup session logging in micro-prefix
LOG_FILE="$WINEPREFIX/last_run.log"
echo "=== Swifter Launch Session: $(date) ===" > "$LOG_FILE"
echo "App: $APP_NAME ($APP_SLUG)" >> "$LOG_FILE"
echo "Executable: $TARGET_EXE" >> "$LOG_FILE"
echo "Backend: $BACKEND" >> "$LOG_FILE"
echo "Wine: $WINE_BIN" >> "$LOG_FILE"
if [ -n "$LAUNCH_ARGS" ]; then
    echo "Launch Arguments: $LAUNCH_ARGS" >> "$LOG_FILE"
fi
echo "=========================================" >> "$LOG_FILE"

# Execute target binary via Wine in the background with configured launch arguments
if [ -n "$LAUNCH_ARGS" ]; then
    "$WINE_BIN" "$TARGET_EXE" $LAUNCH_ARGS "$@" >> "$LOG_FILE" 2>&1 &
else
    "$WINE_BIN" "$TARGET_EXE" "$@" >> "$LOG_FILE" 2>&1 &
fi
WINE_MAIN_PID=$!

sleep 1

# Monitor Wine prefix session: wait while any process in this prefix is running
# This ensures updaters/launchers that spawn a game process and exit do not terminate the game
if [ -n "$WINESERVER_BIN" ] && [ -x "$WINESERVER_BIN" ]; then
    while "$WINESERVER_BIN" -k0 >/dev/null 2>&1; do
        sleep 1
    done
else
    wait $WINE_MAIN_PID 2>/dev/null || true
fi
"#);
    script
}
