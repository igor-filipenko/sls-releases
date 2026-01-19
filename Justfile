default: build

help:
    #!/usr/bin/env bash
    echo "📋 Available commands:"
    echo ""
    echo "  just                      Build the project (same as 'just build')"
    echo "  TOKEN=<token> just run    Run the application with GitHub token"
    echo "  just build                Build the project with optional token"
    echo "  just clean                Clean build artifacts"
    echo "  just setup                Install required dependencies (Java 17, Gradle 8.6)"
    echo "  TOKEN=<token> just token  Validate GitHub token is set"
    echo "  just help                 Show this help message"
    echo ""
    echo "📝 Examples:"
    echo "  export TOKEN=your_token_here"
    echo "  just run"
    echo ""
    echo "  # Or pass token directly:"
    echo "  TOKEN=your_token_here just run"
    echo ""
    echo "⚠️  Note: GITHUB_TOKEN is required for build and run commands"
    echo "   It will be passed to Gradle as an environment variable"

token:
    #!/usr/bin/env bash
    set -e

    if [ -z "${TOKEN}" ]; then
        echo "❌ Error: TOKEN environment variable is not set"
        echo ""
        echo "Please set it using one of these methods:"
        echo "1. Export in your shell: export TOKEN=your_token_here"
        echo "2. Run with: TOKEN=your_token_here just token"
        exit 1
    fi
    echo "✅ GITHUB_TOKEN is set"

setup:
    #!/usr/bin/env bash
    set -e

    if [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ] || command -v sdk > /dev/null 2>&1; then
        source "$HOME/.sdkman/bin/sdkman-init.sh" && echo "✅ SDKMAN is already installed"
        sdk install java 17.0.1-open && echo "✅ Java 17 installed"
        sdk install gradle 8.6 && echo "✅ Gradle 8.6 installed"
    else
        echo "Install SDKMAN: curl -s https://get.sdkman.io | bash"
        exit 1
    fi

run: token
    #!/usr/bin/env bash
    set -e

    source "$HOME/.sdkman/bin/sdkman-init.sh"
    sdk use java 17.0.1-open
    sdk use gradle 8.6

    GITHUB_TOKEN="${TOKEN}" gradle run

build:
    #!/usr/bin/env bash
    set -e

    source "$HOME/.sdkman/bin/sdkman-init.sh"
    sdk use java 17.0.1-open
    sdk use gradle 8.6

    gradle build check

clean:
    #!/usr/bin/env bash
    set -e

    source "$HOME/.sdkman/bin/sdkman-init.sh"
    sdk use java 17.0.1-open
    sdk use gradle 8.6

    gradle clean

# Aliases
all: build