# Generate README.md from src/lib.rs doc comments.
# Intra-doc links ([`text`](target) and [`text`]) are stripped to plain code
# spans because GitHub does not understand Rust's intra-doc link syntax.
readme:
    cargo readme \
        | sed 's/\[`\([^`]*\)`\]([^)]*)/`\1`/g' \
        | sed 's/\[`\([^`]*\)`\]/`\1`/g' \
        | sed '/^#\s*$/d' \
        > README.md
