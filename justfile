run: build
    ./build.sh
    ./run.sh

build:
    ./build.sh

run-spec:
    cd deor_specification && bun server.js

update-deor-with-latest:
    curl -sSf https://raw.githubusercontent.com/nathanphoffman/DeorLang/main/setup/update.sh | sh

install-deor:
    curl -sSf https://raw.githubusercontent.com/nathanphoffman/DeorLang/main/setup/install-deor.sh | sh

install-ext:
    #!/bin/sh
    TMP="$(mktemp -d)"
    curl -sL "https://github.com/nathanphoffman/DeorLang/archive/refs/heads/main.tar.gz" | tar xz -C "$TMP"
    code --install-extension "$(ls "$TMP/DeorLang-main/deor-vscode/"*.vsix | tail -1)"
    rm -rf "$TMP"
    echo "Done — reload VS Code window to apply (Ctrl+Shift+P → Reload Window)."

