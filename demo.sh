#!/usr/bin/env bash
# this_file: demo.sh
# Run extensive vruffr examples
# Usage: ./demo.sh [example-name]
#   all         - Run all examples (default)
#   basic       - Basic conversions
#   styles      - Fill style comparison
#   roughness   - Roughness levels
#   adaptive    - Adaptive roughness demo
#   batch       - Batch processing demo
#   realworld   - Real-world SVG conversions (sag.svg, tiger.svg)
#   clean       - Remove output files

set -euo pipefail
cd "$(dirname "$0")"

VRUFFR="./target/release/vruffr"
OUTPUT_DIR="examples/output"
EXAMPLES_DIR="examples"

# Ensure binary exists
if [[ ! -f "$VRUFFR" ]]; then
    echo "Building vruffr..."
    cargo build --release -p vruffr-cli
fi

mkdir -p "$OUTPUT_DIR"

log() { echo -e "\033[0;32m==>\033[0m $1"; }

# Create a simple test SVG if examples don't exist
create_test_svg() {
    local file="$EXAMPLES_DIR/demo-shapes.svg"
    if [[ ! -f "$file" ]]; then
        mkdir -p "$EXAMPLES_DIR"
        cat > "$file" << 'EOF'
<svg viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
  <rect x="20" y="20" width="120" height="80" rx="8" fill="#3b82f6" stroke="#1e3a5f" stroke-width="2"/>
  <circle cx="250" cy="60" r="45" fill="#ef4444" stroke="#7f1d1d" stroke-width="2"/>
  <ellipse cx="350" cy="150" rx="35" ry="55" fill="#22c55e" stroke="#166534" stroke-width="2"/>
  <polygon points="60,200 120,280 0,280" fill="#f59e0b" stroke="#92400e" stroke-width="2"/>
  <path d="M 150 200 Q 200 150 250 200 T 350 200" fill="none" stroke="#8b5cf6" stroke-width="3"/>
  <rect x="180" y="220" width="60" height="60" fill="#ec4899" stroke="#9d174d" stroke-width="2"/>
</svg>
EOF
    fi
}

demo_basic() {
    log "Basic conversions"
    create_test_svg

    # PNG output
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/basic-default.png"
    echo "  Created: $OUTPUT_DIR/basic-default.png"

    # SVG output
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/basic-default.svg"
    echo "  Created: $OUTPUT_DIR/basic-default.svg"

    # Transparent background
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/basic-transparent.png" --background transparent
    echo "  Created: $OUTPUT_DIR/basic-transparent.png"

    # Scaled output
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/basic-2x.png" --scale 2.0
    echo "  Created: $OUTPUT_DIR/basic-2x.png"
}

demo_styles() {
    log "Fill style comparison"
    create_test_svg

    # Crosshatch (default)
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-crosshatch.png" \
        --fill-style crosshatch
    echo "  Created: $OUTPUT_DIR/style-crosshatch.png"

    # Hachure
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-hachure.png" \
        --fill-style hachure
    echo "  Created: $OUTPUT_DIR/style-hachure.png"

    # Hachure with different angles
    for angle in 0 30 45 90; do
        $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-hachure-${angle}deg.png" \
            --fill-style hachure --hachure-angle "$angle"
        echo "  Created: $OUTPUT_DIR/style-hachure-${angle}deg.png"
    done

    # Different gaps
    for gap in 2 4 8; do
        $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-gap-${gap}.png" \
            --fill-style hachure --hachure-gap "$gap"
        echo "  Created: $OUTPUT_DIR/style-gap-${gap}.png"
    done

    # Strokes only
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-strokes-only.png" --no-fill
    echo "  Created: $OUTPUT_DIR/style-strokes-only.png"

    # Fills only
    $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/style-fills-only.png" --no-stroke
    echo "  Created: $OUTPUT_DIR/style-fills-only.png"
}

demo_roughness() {
    log "Roughness levels"
    create_test_svg

    for r in 0.0 0.5 1.0 1.5 2.0 3.0 5.0; do
        $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/rough-${r}.png" \
            --roughness "$r" --seed 42
        echo "  Created: $OUTPUT_DIR/rough-${r}.png"
    done

    # Bowing variations
    for b in 0.0 1.0 2.0 3.0; do
        $VRUFFR "$EXAMPLES_DIR/demo-shapes.svg" -o "$OUTPUT_DIR/bow-${b}.png" \
            --roughness 1.5 --bowing "$b" --seed 42
        echo "  Created: $OUTPUT_DIR/bow-${b}.png"
    done
}

demo_adaptive() {
    log "Adaptive roughness demo"

    # Create mixed-size SVG
    cat > "$EXAMPLES_DIR/demo-mixed-sizes.svg" << 'EOF'
<svg viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
  <!-- Large shapes -->
  <rect x="20" y="20" width="180" height="120" fill="#3b82f6" stroke="#1e3a5f" stroke-width="2"/>
  <!-- Medium shapes -->
  <circle cx="280" cy="80" r="40" fill="#ef4444" stroke="#7f1d1d" stroke-width="2"/>
  <rect x="320" y="40" width="60" height="60" fill="#22c55e" stroke="#166534" stroke-width="1"/>
  <!-- Small shapes (icons) -->
  <rect x="30" y="180" width="20" height="20" fill="#f59e0b" stroke="#92400e" stroke-width="1"/>
  <circle cx="80" cy="190" r="10" fill="#8b5cf6" stroke="#5b21b6" stroke-width="1"/>
  <rect x="100" y="180" width="15" height="15" fill="#ec4899" stroke="#9d174d" stroke-width="1"/>
  <!-- Tiny shapes -->
  <circle cx="140" cy="188" r="5" fill="#14b8a6" stroke="#115e59" stroke-width="0.5"/>
  <rect x="155" y="183" width="10" height="10" fill="#f97316" stroke="#c2410c" stroke-width="0.5"/>
</svg>
EOF

    # Without adaptive
    $VRUFFR "$EXAMPLES_DIR/demo-mixed-sizes.svg" -o "$OUTPUT_DIR/adaptive-off.png" \
        --roughness 2.0 --adaptive-strength 0.0
    echo "  Created: $OUTPUT_DIR/adaptive-off.png (no adaptive)"

    # With adaptive
    for strength in 0.5 1.0 2.0; do
        $VRUFFR "$EXAMPLES_DIR/demo-mixed-sizes.svg" -o "$OUTPUT_DIR/adaptive-${strength}.png" \
            --roughness 2.0 --adaptive-strength "$strength" --reference-size 100
        echo "  Created: $OUTPUT_DIR/adaptive-${strength}.png (strength=$strength)"
    done
}

demo_batch() {
    log "Batch processing demo"

    # Create several input SVGs
    for i in 1 2 3; do
        cat > "$EXAMPLES_DIR/batch-input-$i.svg" << EOF
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <circle cx="50" cy="50" r="$((20 + i*10))" fill="hsl($((i*120)), 70%, 50%)" stroke="#333" stroke-width="2"/>
</svg>
EOF
    done

    # Process batch
    for svg in "$EXAMPLES_DIR"/batch-input-*.svg; do
        name=$(basename "$svg" .svg)
        $VRUFFR "$svg" -o "$OUTPUT_DIR/${name}-sketch.png" --seed 42 -q
        echo "  Created: $OUTPUT_DIR/${name}-sketch.png"
    done
}

demo_real_world() {
    log "Real-world SVG conversions (sag.svg, tiger.svg)"

    # Find sag.svg and tiger.svg in common locations
    local sag_svg=""
    local tiger_svg=""
    
    for loc in "." "$EXAMPLES_DIR" "examples"; do
        [[ -f "$loc/sag.svg" ]] && sag_svg="$loc/sag.svg"
        [[ -f "$loc/tiger.svg" ]] && tiger_svg="$loc/tiger.svg"
    done

    # Convert sag.svg variants
    if [[ -n "$sag_svg" && -f "$sag_svg" ]]; then
        log "Converting sag.svg"
        
        # Default conversion
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-default.png" --seed 42
        echo "  Created: $OUTPUT_DIR/sag-default.png"
        
        # Different roughness levels
        for r in 0.5 1.5 2.5; do
            $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-rough-${r}.png" \
                --roughness "$r" --seed 42
            echo "  Created: $OUTPUT_DIR/sag-rough-${r}.png"
        done
        
        # Different fill styles
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-crosshatch.png" \
            --fill-style crosshatch --seed 42
        echo "  Created: $OUTPUT_DIR/sag-crosshatch.png"
        
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-hachure.png" \
            --fill-style hachure --seed 42
        echo "  Created: $OUTPUT_DIR/sag-hachure.png"
        
        # Transparent background
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-transparent.png" \
            --background transparent --seed 42
        echo "  Created: $OUTPUT_DIR/sag-transparent.png"
        
        # SVG output
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-sketch.svg" --seed 42
        echo "  Created: $OUTPUT_DIR/sag-sketch.svg"
        
        # Adaptive roughness examples
        log "  Adaptive roughness with sag.svg"
        for strength in 0.5 1.0 2.0; do
            $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-adaptive-${strength}.png" \
                --roughness 2.0 --adaptive-strength "$strength" \
                --reference-size 100 --seed 42
            echo "    Created: $OUTPUT_DIR/sag-adaptive-${strength}.png (strength=$strength)"
        done
        
        # Deduplication examples
        log "  Deduplication with sag.svg"
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-dedup.png" \
            --deduplicate --seed 42
        echo "    Created: $OUTPUT_DIR/sag-dedup.png (with deduplication)"
        
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-dedup-epsilon-0.5.png" \
            --deduplicate --dedup-epsilon 0.5 --seed 42
        echo "    Created: $OUTPUT_DIR/sag-dedup-epsilon-0.5.png (epsilon=0.5)"
        
        # Combined adaptive + deduplication
        log "  Combined adaptive + deduplication with sag.svg"
        $VRUFFR "$sag_svg" -o "$OUTPUT_DIR/sag-adaptive-dedup.png" \
            --roughness 2.0 --adaptive-strength 1.5 \
            --reference-size 100 --deduplicate --seed 42
        echo "    Created: $OUTPUT_DIR/sag-adaptive-dedup.png (adaptive + dedup)"
    else
        echo "  Warning: sag.svg not found (checked: ., examples/, examples)"
    fi

    # Convert tiger.svg variants
    if [[ -n "$tiger_svg" && -f "$tiger_svg" ]]; then
        log "Converting tiger.svg"
        
        # Default conversion
        $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-default.png" --seed 42
        echo "  Created: $OUTPUT_DIR/tiger-default.png"
        
        # Different roughness levels
        for r in 0.5 1.5 2.5; do
            $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-rough-${r}.png" \
                --roughness "$r" --seed 42
            echo "  Created: $OUTPUT_DIR/tiger-rough-${r}.png"
        done
        
        # Different fill styles
        $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-crosshatch.png" \
            --fill-style crosshatch --seed 42
        echo "  Created: $OUTPUT_DIR/tiger-crosshatch.png"
        
        $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-hachure.png" \
            --fill-style hachure --seed 42
        echo "  Created: $OUTPUT_DIR/tiger-hachure.png"
        
        # Transparent background
        $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-transparent.png" \
            --background transparent --seed 42
        echo "  Created: $OUTPUT_DIR/tiger-transparent.png"
        
        # SVG output
        $VRUFFR "$tiger_svg" -o "$OUTPUT_DIR/tiger-sketch.svg" --seed 42
        echo "  Created: $OUTPUT_DIR/tiger-sketch.svg"
    else
        echo "  Warning: tiger.svg not found (checked: ., examples/, examples)"
    fi
}

demo_clean() {
    log "Cleaning output files"
    rm -rf "$OUTPUT_DIR"
    rm -f "$EXAMPLES_DIR"/demo-*.svg "$EXAMPLES_DIR"/batch-*.svg
    echo "  Cleaned"
}

demo_all() {
    demo_basic
    demo_styles
    demo_roughness
    demo_adaptive
    demo_batch
    demo_real_world
    echo ""
    log "All demos complete! Output in: $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/" | head -20
}

# Main
case "${1:-all}" in
    basic)     demo_basic ;;
    styles)    demo_styles ;;
    roughness) demo_roughness ;;
    adaptive)  demo_adaptive ;;
    batch)     demo_batch ;;
    realworld) demo_real_world ;;
    clean)     demo_clean ;;
    all)       demo_all ;;
    *)
        echo "Usage: $0 [basic|styles|roughness|adaptive|batch|realworld|clean|all]"
        exit 1
        ;;
esac
