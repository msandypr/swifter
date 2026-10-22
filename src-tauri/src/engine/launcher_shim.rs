use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwifterAppConfig {
    pub app_name: String,
    pub app_slug: String,
    pub target_exe_relative: String,
    pub backend: String, // "DXVK" | "D3DMetal" | "Vanilla"
    pub enable_msync: bool,
    pub enable_esync: bool,
    pub enable_retina: bool,
    pub custom_env: std::collections::HashMap<String, String>,
    pub wine_custom_path: Option<String>,
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
    osascript -e 'display alert "Wine Runtime Not Found" message "Swifter needs a Wine runtime to run this application.\n\nPlease install Wine or use Swifter Requirements tab." as critical'
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
export WINEDEBUG="-all"

if [ "$ENABLE_MSYNC" = "1" ]; then
    export WINEMSYNC=1
fi

if [ "$ENABLE_ESYNC" = "1" ]; then
    export WINEESYNC=1
fi

if [ "$BACKEND" = "D3DMetal" ]; then
    export WINEESYNC=1
    export WINEMSYNC=1
    export WINEDLLOVERRIDES="d3d12=n;d3d11=n;dxgi=n;${WINEDLLOVERRIDES}"
elif [ "$BACKEND" = "DXVK" ]; then
    export WINEDLLOVERRIDES="d3d11=n,b;d3d10core=n,b;d3d9=n,b;dxgi=n,b;${WINEDLLOVERRIDES}"
fi

if [ "$ENABLE_RETINA" = "1" ]; then
    export WINE_ENABLE_RETINA=1
fi
"#);

    // Custom environment variables from config
    for (k, v) in &config.custom_env {
        script.push_str(&format!("export {}=\"{}\"\n", k, v));
    }

    // Launch execution
    script.push_str(r#"
# Resolve executable path (checks game payload, prefix root, or prefix drive_c)
TARGET_EXE=""
if [ -f "$PAYLOAD_DIR/$EXE_REL" ]; then
    TARGET_EXE="$PAYLOAD_DIR/$EXE_REL"
elif [ -f "$WINEPREFIX/$EXE_REL" ]; then
    TARGET_EXE="$WINEPREFIX/$EXE_REL"
elif [ -f "$WINEPREFIX/drive_c/$EXE_REL" ]; then
    TARGET_EXE="$WINEPREFIX/drive_c/$EXE_REL"
elif [ -f "$EXE_REL" ]; then
    TARGET_EXE="$EXE_REL"
fi

if [ -z "$TARGET_EXE" ] || [ ! -f "$TARGET_EXE" ]; then
    osascript -e "display alert \"Game Executable Missing\" message \"Could not find executable: $EXE_REL\nSearched in payload and prefix.\" as critical"
    exit 1
fi

# Set working directory to executable folder so assets load relatively
cd "$(dirname "$TARGET_EXE")"

exec "$WINE_BIN" "$TARGET_EXE" "$@"
"#);

    script
}
