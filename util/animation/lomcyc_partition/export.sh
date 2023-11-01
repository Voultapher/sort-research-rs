#!/usr/bin/env bash

set -euo pipefail

function export() {
    local format="$1"
    local x_res="$2"
    local y_res="$3"
    local fps="$4"
    local name="$5"

    manim --format="${format}" --resolution=${x_res},${y_res} --fps="${fps}" --media_dir=media --output_file="${name}.${format}" scene.py "${name}"

    cp "media/videos/scene/"${y_res}p${fps}"/${name}.${format}" "${name}.${format}"
}

# export "gif" "640" "360" "20" $1
export "mp4" "1280" "720" "30" $1

# gifski leaves weird artifacts behind swapped bars.
# manim --format=png --resolution=960,540 --fps=20 --media_dir=media scene.py $1
# gifski -o "${1}.gif" media/images/scene/${1}*.png
